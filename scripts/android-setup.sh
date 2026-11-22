#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Tauri Android iskeletini oluşturur ve bu depodaki Android katmanını üzerine
# kopyalar: MainActivity, SAF/paylaşım Kotlin eklentisi, manifest, temalar,
# launcher simgeleri. `tauri android init` Android SDK ister; bu betik SDK
# hazır olduktan sonra bir kez çalıştırılır.
#
# Gerekli ortam:
#   ANDROID_HOME / ANDROID_SDK_ROOT, NDK_HOME (ya da NDK sürümü)
#   JAVA_HOME (JDK 17)
#
# Kullanım: ./scripts/android-setup.sh

set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$(pwd)"
GEN="$ROOT/src-tauri/gen/android"
OVERLAY="$ROOT/android/overlay"

if [[ -z "${ANDROID_HOME:-}${ANDROID_SDK_ROOT:-}" ]]; then
  echo "HATA: ANDROID_HOME veya ANDROID_SDK_ROOT tanımlı değil." >&2
  echo "Android Studio → SDK Manager ile Platform 36 + NDK kurun." >&2
  exit 1
fi

if [[ ! -d "$GEN" ]]; then
  echo "→ tauri android init"
  npx tauri android init
fi

echo "→ Android katmanı kopyalanıyor"
mkdir -p "$GEN/app/src/main/java/com/dod/manager"
cp "$OVERLAY/app/src/main/java/com/dod/manager/MainActivity.kt" "$GEN/app/src/main/java/com/dod/manager/"
cp "$OVERLAY/app/src/main/java/com/dod/manager/AndroidFsPlugin.kt" "$GEN/app/src/main/java/com/dod/manager/"
cp "$OVERLAY/app/src/main/AndroidManifest.xml" "$GEN/app/src/main/AndroidManifest.xml"
mkdir -p "$GEN/app/src/debug"
cp "$OVERLAY/app/src/debug/AndroidManifest.xml" "$GEN/app/src/debug/AndroidManifest.xml"
cp -R "$OVERLAY/app/src/main/res/." "$GEN/app/src/main/res/"
cp -R "$ROOT/src-tauri/icons/." "$GEN/app/src/main/res/drawable/" 2>/dev/null || true

echo "→ Gradle ayarları (minSdk 26, targetSdk 36, documentfile)"
python3 - "$GEN" <<'PY'
import pathlib
import re
import sys

gen = pathlib.Path(sys.argv[1])
gradle = None
for name in ("app/build.gradle.kts", "app/build.gradle"):
    candidate = gen / name
    if candidate.exists():
        gradle = candidate
        break
if gradle is None:
    sys.exit("app/build.gradle(.kts) bulunamadı")

text = gradle.read_text(encoding="utf-8")
text = re.sub(r"(minSdk(?:Version)?\s*=?\s*)\d+", r"\g<1>26", text, count=1)
text = re.sub(r"(targetSdk(?:Version)?\s*=?\s*)\d+", r"\g<1>36", text, count=1)
text = re.sub(r"(compileSdk(?:Version)?\s*=?\s*)\d+", r"\g<1>36", text, count=1)

if "androidx.documentfile" not in text:
    if gradle.suffix == ".kts":
        text += '\n\ndependencies {\n    implementation("androidx.documentfile:documentfile:1.1.0")\n}\n'
    else:
        text += "\n\ndependencies {\n    implementation 'androidx.documentfile:documentfile:1.1.0'\n}\n"

gradle.write_text(text, encoding="utf-8")
print(f"   {gradle.name} güncellendi")
PY

if [[ -f "$GEN/gradle.properties" ]] && ! grep -q "android.useAndroidX" "$GEN/gradle.properties"; then
  printf '\nandroid.useAndroidX=true\nandroid.enableJetifier=false\n' >> "$GEN/gradle.properties"
fi

echo "→ Kotlin eklentisi Rust tarafına kayıtlı: com.dod.manager / AndroidFsPlugin"
echo "→ Ekran görüntüleri: fastlane/metadata/android/*/images/phoneScreenshots"
echo "Bitti. Sıradaki adım: npx tauri android dev"

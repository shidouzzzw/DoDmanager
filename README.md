# Dosya Dolabı

Android için açık kaynak, çevrimdışı dosya yöneticisi. Tauri 2 + Rust + Svelte 5 ile
yazıldı; sahte veri yok, her işlem telefonun gerçek dosya sisteminde çalışır.

[![CI](https://github.com/shidouzzzw/DoDmanager/actions/workflows/ci.yml/badge.svg)](https://github.com/shidouzzzw/DoDmanager/actions/workflows/ci.yml)
[![Lisans: GPL-3.0-or-later](https://img.shields.io/badge/lisans-GPL--3.0--or--later-blue.svg)](LICENSE)

## Özet (mimari, erişim, lisans, tasarım)

1. **Katmanlar:** Svelte arayüzü → `src/lib/services` (invoke sarmalayıcıları) → Rust
   Tauri komutları → `StorageProvider`. Arayüz hiçbir zaman dosya sistemine dokunmaz;
   `fs` eklentisi kayıtlıdır ama webview'a dosya kapsamı **verilmez**.
2. **Sağlayıcılar:** `LocalProvider` (mutlak yol, `std::fs`) ve `SafProvider` (SAF
   `content://` URI'leri, Kotlin köprüsü). İkisi de "hedef klasör + ad" API'si kullanır;
   SAF URI'si taklit edilmez, oluşturma işlemleri Kotlin tarafında yapılır.
3. **Erişim modeli:** Varsayılan **Scoped Storage** (kullanıcı SAF ile klasör seçer,
   `takePersistableUriPermission` ile kalıcı izin). İsteğe bağlı **Tüm dosyalara erişim**
   (`MANAGE_EXTERNAL_STORAGE`); ilk açılışta iki mod anlatılır, Ayarlar'dan değiştirilir.
4. **Uzun işler:** kopyalama/taşınma/silme/zip işleri `spawn_blocking` üzerinde çalışır,
   `job:progress` / `job:finished` eventleri yayınlar ve `cancel_job` ile iptal edilir.
5. **Arama:** ayrı iş parçacığında, 250 ms'lik partiler hâlinde `search:results` eventi
   ile akıtılır; yeni sorgu öncekini iptal eder.
6. **Küçük resimler:** Rust `image` crate, disk önbelleği (`app_cache_dir/thumbnails`),
   liste görünür öğeler için tembel üretir; SAF belgeleri önce önbelleğe indirilir.
7. **Kotlin eklentisi:** SAF seçici, paylaşım (`ACTION_SEND`), "Birlikte aç"
   (`ACTION_VIEW`), gelen intent, depolama bilgisi, duvar kağıdı vurgu rengi ve sistem
   çubuğu simgeleri. Rust `register_android_plugin` ile bağlanır.
8. **Lisans:** GPL-3.0-or-later. Kapalı SDK, reklam, telemetri yok; **INTERNET izni
   yok** (yalnızca debug derlemesinde Vite için). F-Droid uyumlu, yeniden üretilebilir.
9. **Tasarım:** "kâğıt ve mürekkep" dili — sıcak kâğıt zemin, mürekkep metin, tek vurgu
   rengi, dolap/çekmece motifi; Bricolage Grotesque + Inter (ikisi OFL, gömülü).
10. **Tema:** CSS değişkenleri, açık/koyu/sistem, Android 12+ dinamik vurgu rengi,
    edge-to-edge + `env(safe-area-inset-*)`, klavye açılışı `visualViewport` ile.
11. **Erişilebilirlik:** 48dp dokunma hedefleri, TalkBack etiketleri, sistem yazı
    boyutu (rem), `prefers-reduced-motion` desteği, yüksek kontrastlı token'lar.
12. **Test:** Rust birim testleri (sağlayıcı, util, hata eşleme), frontend servis
    testleri (Vitest), `cargo fmt`/`clippy -D warnings`, ESLint + Prettier, CI'da APK.

## Özellikler

- **Ana ekran:** eklenen konumlar (dahili depolama, SD kart, kullanıcı klasörleri),
  favoriler, son kullanılanlar, uygulama içi çöp kutusu ve depolama özeti.
- **Gezinme:** breadcrumb, Android geri tuşu ve hareket geri (JS köprüsü üzerinden),
  çökmeyen uzun yol kırpma, kök algılama.
- **Görünüm:** ızgara/liste, ada/tarihe/boyuta/türe göre sıralama, türe göre filtre
  (görsel, video, belge, ses, arşiv), gizli dosyaları gösterme.
- **Seçim:** uzun basma ile seçim modu, parmak kaydırarak aralık seçimi, tümünü seç,
  seçim çubuğunda paylaş/kopyala/taşı/sil/zip/özellikler.
- **İşlemler:** yeniden adlandırma, kopyalama, taşıma (hedef klasör seçici), silme
  (çöpe), kalıcı silme, klasör/dosya oluşturma, zip oluşturma ve çıkarma.
- **Tablet/katlanabilir:** uzun basıp klasörün üzerine sürükleyerek taşıma.
- **Önizleme:** görsel (pinch-zoom, çift dokunuş), video/ses, PDF (pdf.js, sayfa
  gezinme), metin/kod (sözdizimi renklendirme), markdown; aynı klasörde sağa/sola
  kaydırarak dosya değiştirme.
- **Sistem entegrasyonu:** paylaşım menüsü, "Birlikte aç", diğer uygulamalardan gelen
  `ACTION_VIEW`/`ACTION_SEND` dosyalarını içe aktarma.
- **Geri al:** silme, taşıma ve yeniden adlandırma için snackbar'dan geri alma.
- **Haptik:** seçim, sürükleme, silme ve başarıda titreşim (ayardan kapatılabilir).
- **Büyük klasörler:** sanal liste; yalnızca görünür satırlar DOM'a girer.
- **Hatalar:** Türkçe, kod tabanlı mesajlar ve çözüm butonları ("Ayarlara git",
  "Yeniden dene", "Klasör seç").

## Ekran görüntüleri

Ekran görüntüleri `fastlane/metadata/android/{tr-TR,en-US}/images/phoneScreenshots/`
altına eklenir (F-Droid ve mağaza metadata'sı aynı klasörü kullanır). Depoda örnek
görsel tutulmaz; uygulama çalıştırıldıktan sonra gerçek ekran görüntüleri eklenmelidir.

## Kurulum

### Gereksinimler

| Araç                 | Sürüm                                 |
| -------------------- | ------------------------------------- |
| Node.js              | ≥ 20.19 (npm 11)                      |
| Rust                 | ≥ 1.85 (kararlı)                      |
| JDK                  | 17                                    |
| Android SDK Platform | 36                                    |
| Android Build Tools  | 36.0.0                                |
| Android NDK          | 27.2.12479018 (r27c)                  |
| Tauri CLI            | `@tauri-apps/cli` 2.x (devDependency) |

`ANDROID_HOME` (veya `ANDROID_SDK_ROOT`), `NDK_HOME` ve `JAVA_HOME` tanımlı olmalıdır.
Tauri Android hedefi için ayrıca `rustup target add aarch64-linux-android
armv7-linux-androideabi i686-linux-android x86_64-linux-android` gerekir.

### Adımlar

```bash
npm ci
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

# Android iskeletini oluşturur, MainActivity/manifest/tema/ikonları kopyalar,
# minSdk 26 / targetSdk 36 ve androidx.documentfile bağımlılığını uygular.
./scripts/android-setup.sh

# Geliştirme (Vite + cihaz/emülatör)
npm run tauri android dev

# Release APK (imzasız ya da keystore.properties varsa imzalı)
npm run tauri android build -- --apk
```

APK çıktısı: `src-tauri/gen/android/app/build/outputs/apk/universal/release/`.

### İmzalama

Anahtarı **repoya koymayın**. `src-tauri/gen/android/keystore.properties` dosyasını
yerelde oluşturun (bu dosya `.gitignore` içindedir):

```properties
storeFile=/absolute/yol/release.jks
storePassword=...
keyAlias=dod
keyPassword=...
```

CI için GitHub → Settings → Secrets and variables → Actions altında
`ANDROID_KEY_BASE64` (base64 keystore), `ANDROID_KEY_ALIAS`, `ANDROID_STORE_PASSWORD`,
`ANDROID_KEY_PASSWORD` tanımlayın; `release.yml` bunları geçici bir
`keystore.properties` dosyasına yazar ve iş sonunda siler. Secret yoksa iş imzasız APK
üretir; APK yine Release'e eklenir.

`keystore.properties` üretimi:

```bash
keytool -genkeypair -v -keystore release.jks -alias dod \
  -keyalg RSA -keysize 4096 -validity 10000
base64 -w0 release.jks   # ANDROID_KEY_BASE64
```

### İzinler

`android/overlay/app/src/main/AndroidManifest.xml` içinde:

| İzin                                          | Amaç                                         |
| --------------------------------------------- | -------------------------------------------- |
| `READ_EXTERNAL_STORAGE` (`maxSdkVersion=32`)  | Android 10–12'de geniş okuma                 |
| `WRITE_EXTERNAL_STORAGE` (`maxSdkVersion=29`) | Android 10'da "tüm dosyalar" modu            |
| `MANAGE_EXTERNAL_STORAGE`                     | Android 11+ isteğe bağlı "tüm dosyalar" modu |

**INTERNET izni yoktur.** Yalnızca `src/debug/AndroidManifest.xml` debug derlemesine
INTERNET ve cleartext ekler (Vite geliştirme sunucusu için). Release APK çevrimdışıdır.
`FileProvider` yalnızca paylaşım/Birlikte aç için `cache-path` ve `files-path` açar.

## Android 10–15 davranış farkları

| Sürüm   | Davranış                                                                            | Uygulamadaki karşılığı                                           |
| ------- | ----------------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| 10 (29) | Scoped storage zorunlu; `requestLegacyExternalStorage` yalnızca bu sürümde etkili   | Varsayılan SAF; "tüm dosyalar" modunda legacy bayrak açık        |
| 11 (30) | `MANAGE_EXTERNAL_STORAGE` geldi; `Android/data` ve `Android/obb` SAF ile de kısıtlı | Tüm dosyalar modu ayarlardan; kısıtlı klasörlerde anlaşılır hata |
| 12 (31) | Dinamik renk (`system_accent1_400`), splash screen                                  | Vurgu rengi seçeneği; tema token'ları                            |
| 13 (33) | Medya izinleri ayrıldı; bildirim izni                                               | Medya için izin istenmez, SAF kullanılır                         |
| 14 (34) | Kısmi medya erişimi (`READ_MEDIA_VISUAL_USER_SELECTED`)                             | SAF kalıcı izinleri tercih edilir                                |
| 15 (35) | Edge-to-edge zorunlu, 16 KB sayfa boyutu                                            | `WindowCompat` + CSS `env()`; NDK r27 / Rust 1.85+ 16 KB uyumlu  |

## Erişim modelleri ve sınırlar

- **Scoped (varsayılan):** kullanıcı bir klasör seçer; yol `content://…/tree/…` olarak
  saklanır. Uygulama yalnızca verilen ağacı görür. Ağaç dışına taşıma yapılamaz;
  hedef seçici yalnızca erişilebilir kökleri listeler.
- **Tüm dosyalar (isteğe bağlı):** `MANAGE_EXTERNAL_STORAGE` açıksa mutlak yollar
  kullanılır, tüm birimler listelenir. Play Store bu izni kısıtlar; F-Droid ve yan
  yükleme için uygundur. İzin geri alınırsa uygulama otomatik olarak Scoped'a düşer ve
  kullanıcıya bildirir.
- **Bilinçli sınır:** SAF üzerinde rastgele erişim (seek) yoktur; PDF/video önizlemesi
  için dosya önbelleğe kopyalanır. Çok büyük dosyalarda bu disk maliyeti yaratır;
  önbellek `app_cache_dir` altında tutulur ve sistem tarafından temizlenebilir.

## Mimari ve klasör yapısı

```
.
├─ android/overlay/                 # tauri android init sonrası kopyalanan katman
│  └─ app/src/main/…                # MainActivity, SAF Kotlin eklentisi, manifest, res
├─ fastlane/metadata/android/       # F-Droid/mağaza metinleri (tr-TR, en-US)
├─ fdroid/metadata/                 # F-Droid build recipe
├─ scripts/
│  ├─ android-setup.sh              # init + overlay + gradle ayarları
│  ├─ make-icons.py                 # launcher/Tauri ikonları (Pillow)
│  └─ build-icons.mjs               # Lucide SVG → src/lib/icons/paths.ts
├─ src/                             # Svelte arayüzü
│  ├─ App.svelte                    # ekran yönlendirme, geri tuşu, tema, event'ler
│  ├─ app.css                       # tema token'ları, tipografi, yardımcı sınıflar
│  └─ lib/
│     ├─ components/                # Home, Browser, Preview, Settings, Trash, Picker…
│     ├─ services/                  # backend, files, settings, search, jobs, haptics
│     ├─ state/app.svelte.ts        # merkezî runes durumu
│     ├─ i18n/locales/{tr,en}.json  # tüm kullanıcı metinleri
│     └─ ui/                        # dosya türü, sıralama, renklendirme
└─ src-tauri/
   ├─ src/commands.rs               # Tauri komutları (tek giriş noktası)
   ├─ src/provider/{local,saf}.rs   # StorageProvider uygulamaları
   ├─ src/{transfer,trash,archive,search,thumbnails}.rs
   ├─ src/{jobs,settings,mobile,error,models,util}.rs
   ├─ capabilities/android.json     # Android yetkileri
   └─ tauri.conf.json
```

Veri akışı: bileşen → `services/*.ts` → `invoke` → `commands.rs` → `provider::open()`
→ `LocalProvider` veya `SafProvider` → Kotlin (yalnızca SAF) → Android.

## Geliştirme

```bash
npm run dev          # yalnızca arayüz (tarayıcıda mock yok; Tauri API'leri gerekir)
npm run check        # svelte-check + TypeScript
npm run lint         # ESLint (Svelte + TS)
npm run test         # Vitest
npm run format       # Prettier
cd src-tauri
cargo fmt --check
cargo clippy --target aarch64-linux-android --lib -- -D warnings
cargo test
```

Rust testleri Tauri'ye bağımlı olmayan çekirdeği kapsar (`util`, `error`, `models`,
`provider::local`, `trash`, `archive`, `search`). Android'e özgü kod `#[cfg]` ile
ayrıldığı için masaüstünde `cargo test` çalışır; Android SDK/NDK olmadan da
`cargo check --target aarch64-linux-android --lib` ile hedef derleme doğrulanabilir.

## Çeviri

Tüm metinler `src/lib/i18n/locales/*.json` dosyalarındadır; varsayılan dil Türkçe.
Yeni dil eklemek için aynı anahtarlarla bir JSON dosyası oluşturup `src/lib/i18n/index.ts`
içindeki `DICTIONARIES` haritasına ekleyin. `dictionariesInSync()` testi eksik anahtarı
yakalar. Weblate/Crowdin için dosya biçimi düz JSON'dur (iç içe nesne yok).

## F-Droid

1. `fastlane/metadata/android/{tr-TR,en-US}` metinleri ve ekran görüntüleri hazırdır.
2. `fdroid/metadata/com.dod.manager.yml` dosyasını `fdroiddata` deposuna
   `metadata/com.dod.manager.yml` olarak kopyalayın.
3. `fdroid build com.dod.manager` ile deneyin; tarif NDK r27c, JDK 17 ve Node 20 ister.
4. Sürüm etiketi (`v0.1.0`) F-Droid'in "Auto Update Mode: Version" satırıyla eşleşir.

Tarif kaynaktan derler: `npm ci` → `tauri android build --apk`. Kapalı ikili yoktur;
fontlar (OFL) ve ikonlar (ISC) repoda, `pdfjs-dist` Apache-2.0'dır.

## Lisans

GPL-3.0-or-later. Gerekçe: dosya yöneticisi sistem düzeyinde güçlü izinler kullanır;
kullanıcının çalıştırdığı yazılımı inceleyip değiştirebilmesi ve türev çalışmaların da
özgür kalması bu güven için gereklidir. Ayrıntı: [LICENSE](LICENSE),
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

Katkı: [CONTRIBUTING.md](CONTRIBUTING.md) · Davranış: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) ·
Güvenlik: [SECURITY.md](SECURITY.md) · Sürümler: [CHANGELOG.md](CHANGELOG.md)

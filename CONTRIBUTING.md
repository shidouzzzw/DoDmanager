# Katkı Rehberi

Dosya Dolabı'na katkı sağladığınız için teşekkürler. Proje GPL-3.0-or-later ile
lisanslıdır; katkılarınız aynı lisans altında yayımlanır.

## Geliştirme ortamı

Gereksinimler ve kurulum adımları [README.md](README.md) içindedir. Kısa özet:

```bash
npm ci
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
./scripts/android-setup.sh
npm run tauri android dev
```

Android SDK 36, NDK r27b, JDK 17 ve Rust 1.85+ gerekir. `ANDROID_HOME`, `NDK_HOME`,
`JAVA_HOME` tanımlı olmalıdır.

## Kod stili

- **Rust:** `cargo fmt`, `cargo clippy -- -D warnings`. Hata dönen fonksiyonlar
  `AppResult<T>` kullanır; yeni hata kodları `error.rs` içindeki `AppError`'a eklenir.
  Tauri'ye bağımlı olmayan mantık `provider/`, `util.rs`, `trash.rs`, `archive.rs`
  içinde tutulur ki masaüstünde test edilebilsin.
- **TypeScript/Svelte:** ESLint + Prettier. Bileşenler servisleri doğrudan çağırmaz;
  `src/lib/services` üzerinden gider. Kullanıcıya görünen metin **yalnızca**
  `src/lib/i18n/locales/*.json` içine yazılır.
- **Kotlin:** Android katmanı `android/overlay/` altında tutulur; `tauri android init`
  çıktısı (`src-tauri/gen/`) sürüm kontrolüne girmez.
- Yorumlar kısa ve Türkçe; karmaşık algoritmalarda "neden" açıklanır.
- Her kaynak dosyanın ilk satırı `SPDX-License-Identifier: GPL-3.0-or-later` olmalıdır.
  Üçüncü taraf dosyalar kendi SPDX kimliğini korur.

## Değişiklik akışı

1. Konuyu belirten bir issue açın (hata ise yeniden üretme adımlarıyla).
2. `main` üzerinden kısa ömürlü bir dal açın: `feat/…`, `fix/…`, `docs/…`.
3. Değişikliği küçük ve odaklı tutun; ilgisiz yeniden biçimlendirme yapmayın.
4. Testleri çalıştırın:

   ```bash
   npm run lint && npm run check && npm run test
   cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
   ```

5. PR açıklamasında neyin neden değiştiğini, testleri ve varsa ekran görüntülerini
   belirtin. UI değişikliklerinde açık/koyu tema ve 200% yazı boyutu kontrol edilir.

## Yeni dil ekleme

`src/lib/i18n/locales/tr.json` dosyasını kopyalayıp çevirin, `src/lib/i18n/index.ts`
içindeki `DICTIONARIES` haritasına ekleyin. `npm run test` eksik/fazla anahtarı
`dictionariesInSync` testiyle yakalar. Weblate/Crowdin'e hazır düz JSON yapısı vardır.

## Bağımlılık politikası

- Yeni bağımlılık yalnızca gerekliyse eklenir; lisansı GPL-3.0-or-later ile uyumlu
  olmalıdır (MIT, Apache-2.0, BSD, ISC, OFL kabul).
- Kapalı kaynak SDK, reklam, analitik, çökme raporlama ve telemetri eklenmez.
- `INTERNET` izni release manifest'ine eklenmez.
- Yeni bağımlılık `THIRD_PARTY_LICENSES.md` tablosuna işlenir.

## Commit mesajları

`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `build:` önekleri kullanılır; başlık
72 karakteri geçmez. Her commit derlenebilir olmalıdır.

## Davranış

Katkıda bulunan herkes [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) kurallarına uyar.

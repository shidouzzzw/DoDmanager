# Değişiklik Günlüğü

Bu dosya [Keep a Changelog](https://keepachangelog.com/tr/1.1.0/) biçimini izler ve
proje [Semantic Versioning](https://semver.org/lang/tr/) kullanır.

## [Unreleased]

### Eklenecek

- Ekran görüntüleri ve F-Droid mağaza görselleri
- İsteğe bağlı: çoklu pencere (tablet) düzeni

## [0.1.1] - 2026-11-23

### Düzeltildi

- Uygulama açılışta kapanıyordu: Android teması `android:Theme.Material.*`
  tabanlıydı, oysa Tauri'nin `WryActivity` sınıfı `AppCompatActivity`'ten türer ve
  AppCompat `onCreate` içinde temayı doğrular. Tema
  `Theme.AppCompat.Light.NoActionBar` üst temasına taşındı.

## [0.1.0] - 2026-11-22

İlk sürüm. Android 8.0 (API 26) ve üzeri.

### Eklendi

- Tauri 2 + Rust + Svelte 5 tabanlı, tamamen çevrimdışı dosya yöneticisi
- `LocalProvider` ve `SafProvider` üzerinden gerçek dosya sistemi erişimi
- Scoped Storage (SAF, kalıcı izin) ve isteğe bağlı "tüm dosyalara erişim" modu;
  ilk açılışta karşılaştırma ekranı, Ayarlar'dan geçiş
- Ana ekran: konumlar, favoriler, son kullanılanlar, çöp kutusu, depolama özeti
- Izgara/liste görünümü, sıralama (ad/tarih/boyut/tür), tür filtresi, gizli dosyalar
- Uzun basma, sürükleyerek aralık seçimi, tümünü seç; alt aksiyon çubuğu ve
  bottom sheet menüsü
- Yeniden adlandırma, kopyalama, taşıma, çöpe silme, kalıcı silme, klasör/dosya
  oluşturma, zip oluşturma/çıkarma; hedef klasör seçici
- Tablet ve katlanabilirlerde sürükleyip klasöre bırakma
- Önizleme: görsel (pinch-zoom), video, ses, PDF (pdf.js), metin/kod renklendirme,
  markdown; aynı klasörde kaydırarak gezinme
- Sistem paylaşımı, "Birlikte aç", `ACTION_VIEW`/`ACTION_SEND` ile gelen dosyaları
  içe aktarma
- Silme/taşıma/yeniden adlandırma için geri alma snackbar'ı
- Haptik geri bildirim (seçim, sürükleme, silme, başarı)
- 10.000+ öğe için sanal liste
- Rust'ta üretilen, disk önbellekli, tembel küçük resimler
- Ayrı iş parçacığında akan arama; ilerleme event'leri ve iptal
- Türkçe hata mesajları, hata kodları ve çözüm butonları
- "Kâğıt ve mürekkep" tasarım dili; açık/koyu/sistem tema, Android 12+ dinamik renk,
  edge-to-edge ve güvenli alan yönetimi, 150–250 ms animasyonlar, reduced-motion
- Türkçe varsayılan i18n (İngilizce çeviri dâhil), OFL fontlar gömülü
- GPL-3.0-or-later lisansı, SPDX başlıkları, F-Droid metadata'sı, CI ve release
  iş akışları, Dependabot

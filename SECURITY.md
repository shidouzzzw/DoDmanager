# Güvenlik Politikası

## Desteklenen sürümler

| Sürüm | Destek |
| ----- | ------ |
| 0.1.x | Evet   |
| < 0.1 | Hayır  |

## Açık bildirimi

Güvenlik açığını **herkese açık issue olarak açmayın**. GitHub'ın özel güvenlik
bildirimi özelliğini kullanın:

<https://github.com/shidouzzzw/DoDmanager/security/advisories/new>

Bildirimde bulunması yararlı olanlar:

- Etkilenen sürüm ve Android sürümü
- Yeniden üretme adımları (mümkünse en küçük dosya/klasör düzeni)
- Etki: bilgi sızması, yol aşımı, izin yükseltme, hizmet reddi vb.
- Varsa önerilen düzeltme

Yanıt süresi hedefi: 72 saat içinde ilk yanıt, 30 gün içinde düzeltme veya plan.
Düzeltme yayımlandıktan sonra, isterseniz adınızı teşekkür listesine ekleriz.

## Tehdit modeli

Dosya Dolabı çevrimdışıdır (`INTERNET` izni yoktur). Başlıca riskler:

- **Yol aşımı:** Rust tarafında `util::validate_local_path` mutlak yolu ve `..`
  bileşenlerini reddeder; SAF yolları belge URI'si olarak doğrulanır.
- **SAF URI'leri:** Kullanıcı yalnızca verdiği ağaca erişir; `takePersistableUriPermission`
  ile saklanan izinler uygulama ayarlarından kaldırılabilir.
- **FileProvider:** Yalnızca `cache-path` ve `files-path` paylaşılır; `exported=false`
  ve `grantUriPermissions=true` ile geçici okuma izni verilir.
- **WebView:** CSP `default-src 'self'`; `object-src 'none'`, `base-uri 'none'`,
  `form-action 'none'`. Uzak içerik yüklenmez. `pdfjs-dist` yerel paketten gelir.
- **HTML enjeksiyonu:** Önizlemede markdown/kod çıktısı `escapeHtml` ile kaçırılır;
  `{@html}` yalnızca bu iki üretici ve derleme zamanı ikon sabitleri için kullanılır.
- **Zip:** çıkarma sırasında hedef yolları normalize edilir; `..` içeren girdiler
  reddedilir, toplam açılan boyut sınırlandırılır.

## Kapsam dışı

- Kullanıcının cihazında root erişimi gerektiren saldırılar
- Android işletim sistemi veya WebView'in kendi açıkları
- Sosyal mühendislik

## Güvenli yapılandırma

- Release imza anahtarı repoda tutulmaz; `keystore.properties` yerelde ve CI
  secret'larında saklanır.
- `INTERNET` izni yalnızca debug manifest'indedir.
- Bağımlılıklar Dependabot ile izlenir; `npm audit` ve `cargo audit` çalıştırılabilir.

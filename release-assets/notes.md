# Dosya Dolabı v0.1.1

Android 8.0+ (API 26) APK. Kurulabilir ve imzası doğrulanmıştır.

- Paket: `com.dod.manager`
- Sürüm: 0.1.1 (versionCode 2)
- İmza: RSA 4096, `CN=Dosya Dolabi, O=DoD Manager, C=TR`
- APK Signature Scheme v2 + v3 doğrulandı

## Bu sürümde düzeltilen

**Uygulama açılışta kapanıyordu.** Android teması `android:Theme.Material.Light.NoActionBar`
tabanlıydı; Tauri'nin `WryActivity` sınıfı `AppCompatActivity`'ten türediği için AppCompat
açılışta `You need to use a Theme.AppCompat theme (or descendant) with this activity`
istisnasını fırlatıyor ve süreç hemen sonlanıyordu. Tema artık
`Theme.AppCompat.Light.NoActionBar` üst temasını kullanıyor.

Önceki sürüm (v0.1.0) kuruluysa doğrudan güncelleyebilirsiniz; imza anahtarı aynıdır.

Kurulum: APK'yı indirin, "Bilinmeyen kaynaklardan yüklemeye izin ver" seçeneğini açın.

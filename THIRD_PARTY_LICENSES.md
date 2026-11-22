# Üçüncü Taraf Lisansları

Dosya Dolabı GPL-3.0-or-later ile lisanslıdır. Aşağıdaki bağımlılıklar bu lisansla
uyumludur; hiçbiri kapalı kaynak değildir. Tam lisans metinleri ilgili paketlerin
içinde (`node_modules/<paket>/LICENSE`, Cargo kayıt defteri) ve `LICENSES/` altındadır.

## Çalışma zamanı (frontend)

| Paket                        | Sürüm | Lisans              |
| ---------------------------- | ----- | ------------------- |
| `svelte`                     | 5.x   | MIT                 |
| `@tauri-apps/api`            | 2.x   | MIT veya Apache-2.0 |
| `@tauri-apps/plugin-dialog`  | 2.x   | MIT veya Apache-2.0 |
| `@tauri-apps/plugin-haptics` | 2.x   | MIT veya Apache-2.0 |
| `@tauri-apps/plugin-opener`  | 2.x   | MIT veya Apache-2.0 |
| `pdfjs-dist`                 | 6.x   | Apache-2.0          |

## Derleme ve test (frontend)

| Paket                                                                          | Lisans              |
| ------------------------------------------------------------------------------ | ------------------- |
| `vite`                                                                         | MIT                 |
| `@sveltejs/vite-plugin-svelte`                                                 | MIT                 |
| `typescript`                                                                   | Apache-2.0          |
| `svelte-check`                                                                 | MIT                 |
| `vitest`                                                                       | MIT                 |
| `eslint`, `@eslint/js`, `eslint-plugin-svelte`, `typescript-eslint`, `globals` | MIT                 |
| `prettier`, `prettier-plugin-svelte`                                           | MIT                 |
| `@types/node`                                                                  | MIT                 |
| `@tauri-apps/cli`                                                              | MIT veya Apache-2.0 |

## Rust

| Crate                                    | Lisans              |
| ---------------------------------------- | ------------------- |
| `tauri`, `tauri-build`, `tauri-plugin-*` | MIT veya Apache-2.0 |
| `serde`, `serde_json`                    | MIT veya Apache-2.0 |
| `thiserror`                              | MIT veya Apache-2.0 |
| `image`                                  | MIT veya Apache-2.0 |
| `zip`                                    | MIT                 |
| `base64`                                 | MIT veya Apache-2.0 |
| `uuid`                                   | Apache-2.0 veya MIT |
| `sha2`                                   | MIT veya Apache-2.0 |
| `mime_guess`                             | MIT                 |
| `tempfile` (yalnızca test)               | MIT veya Apache-2.0 |

## Android / Kotlin

| Bileşen                               | Lisans              |
| ------------------------------------- | ------------------- |
| Tauri Android (`app.tauri`)           | MIT veya Apache-2.0 |
| AndroidX Core, DocumentFile, Activity | Apache-2.0          |

## Gömülü varlıklar

| Varlık              | Lisans      | Konum                                               |
| ------------------- | ----------- | --------------------------------------------------- |
| Bricolage Grotesque | SIL OFL 1.1 | `src/assets/fonts/` + `OFL-BricolageGrotesque.txt`  |
| Inter               | SIL OFL 1.1 | `src/assets/fonts/` + `OFL-Inter.txt`               |
| Lucide ikonları     | ISC         | `LICENSES/Lucide-ISC.txt`, `src/lib/icons/paths.ts` |

## Yeniden üretme

```bash
npm ls --all --json | node -e "…"        # npm bağımlılık ağacı
cargo tree --manifest-path src-tauri/Cargo.toml
cargo install cargo-about && cargo about generate about.hbs   # isteğe bağlı
npx license-checker --production --summary                     # isteğe bağlı
```

Liste doğrudan bağımlılıkları kapsar; geçişli bağımlılıklar aynı lisans ailelerinden
gelir (MIT/Apache-2.0/BSD/ISC). Yeni bağımlılık eklerken bu dosyayı güncelleyin.

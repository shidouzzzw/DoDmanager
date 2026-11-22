#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Uygulama simgesini ve Android mipmap'lerini üretir.

Simge "kâğıt ve mürekkep" diline uyar: kâğıt zemin üzerinde mürekkep bir
dolap ve vurgu renkli çekmece kulpları. Saf geometrik çizim; metin yok.

Kullanım: python3 scripts/make-icons.py   (Pillow gerekir)
"""

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
PAPER = (244, 239, 228, 255)
INK = (25, 28, 33, 255)
ACCENT = (52, 73, 155, 255)
SIZE = 1024


def draw_mark(size: int) -> Image.Image:
    """Simgeyi `size` px olarak çizer (4x süper örnekleme ile)."""
    scale = 4
    s = size * scale
    img = Image.new("RGBA", (s, s), PAPER)
    d = ImageDraw.Draw(img)

    def rect(x, y, w, h, r, color):
        d.rounded_rectangle(
            [x * s, y * s, (x + w) * s, (y + h) * s],
            radius=r * s,
            fill=color,
        )

    # Dolap gövdesi
    rect(0.20, 0.16, 0.60, 0.68, 0.09, INK)
    # Üst çekmece yüzü ve alt çekmece yüzü
    rect(0.26, 0.23, 0.48, 0.24, 0.045, PAPER)
    rect(0.26, 0.53, 0.48, 0.24, 0.045, PAPER)
    # Kulplar: üstte mürekkep, altta vurgu rengi
    rect(0.42, 0.335, 0.16, 0.028, 0.014, INK)
    rect(0.42, 0.635, 0.16, 0.028, 0.014, ACCENT)
    # Ayaklar
    rect(0.26, 0.84, 0.05, 0.05, 0.012, INK)
    rect(0.69, 0.84, 0.05, 0.05, 0.012, INK)

    return img.resize((size, size), Image.LANCZOS)


def main() -> None:
    icons = ROOT / "src-tauri" / "icons"
    icons.mkdir(parents=True, exist_ok=True)
    master = draw_mark(SIZE)
    master.save(icons / "icon.png")

    for name, px in {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "Square150x150Logo.png": 150,
        "StoreLogo.png": 50,
    }.items():
        master.resize((px, px), Image.LANCZOS).save(icons / name)

    densities = {
        "mdpi": 48,
        "hdpi": 72,
        "xhdpi": 96,
        "xxhdpi": 144,
        "xxxhdpi": 192,
    }
    res = ROOT / "android" / "overlay" / "app" / "src" / "main" / "res"
    for density, px in densities.items():
        folder = res / f"mipmap-{density}"
        folder.mkdir(parents=True, exist_ok=True)
        launcher = master.resize((px, px), Image.LANCZOS)
        launcher.save(folder / "ic_launcher.png")
        launcher.save(folder / "ic_launcher_round.png")
    print("simge ve mipmap'ler üretildi")


if __name__ == "__main__":
    main()

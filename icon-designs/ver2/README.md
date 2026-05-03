# Browser Switcher Icons

Two color variants of the "Stacked Discs" icon for Browser Switcher.

## Folders
- icons/        Dark variant (dark tile, screen-blended discs)
- icons-light/  Light variant (white tile, multiply-blended discs)

## Sizes
Each folder contains:
  16x16.png   tray / menu / legacy
  20x20.png   Windows UI scale
  24x24.png   small UI
  32x32.png   taskbar / shell
  40x40.png   Windows scale
  48x48.png   shell
  64x64.png   high DPI
  128x128.png app metadata
  128x128_2x.png  → rename to 128x128@2x.png on disk
  256x256.png Windows icon
  512x512.png store / marketing master
  icon.png    512x512 (Tauri default)
  icon.svg    master vector source

## Notes
- 16/20/24/32px keep the overlapping-disc design (rasterized from the master SVG).
- 40-512px are high-quality SVG rasterization.
- File "128x128_2x.png" is the @2x retina asset; rename to include "@" once placed.

## Generating icon.ico (multi-size)
Using ImageMagick:

  magick 16x16.png 24x24.png 32x32.png 48x48.png 64x64.png 128x128.png 256x256.png icon.ico

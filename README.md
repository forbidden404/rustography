## Usage
```console
Execute a command for a specific image

Usage: rustography --path <PATH> <COMMAND>

Commands:
  add-border
          Add a white border to an image
  fill-to-aspect-ratio
          Fill the image with white to fit a given aspect ratio
  fill-to-aspect-ratio-with-border
          Fill the image with white to fit a given aspect ratio and add white border
  add-caption
          Add text on the bottom with camera, focal length, aperture, shutter speed and ISO
  fill-to-aspect-ratio-with-border-and-caption
          Fill the image with white to fit a given aspect ratio, add white border and add text on the bottom with camera, focal length, aperture, shutter speed and ISO
  help
          Print this message or the help of the given subcommand(s)

Options:
  -p, --path <PATH>  The path to the image
  -h, --help         Print help
  -V, --version      Print version
```

## Requirements
- [ImageMagick](https://imagemagick.org/script/convert.php)

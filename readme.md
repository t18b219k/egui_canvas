# egui Canvas2D painter.
 
## target user
 * webgl not available.
 * avoid crash glow painter.
 * want to support almost modern browser.
## not for
 * 1 want to use epaint::Mesh,
 * 2 fine text quality.
 * 3 bracingly fast startup.

## technical restriction in Canvas2D
 * 1,2 Canvas2D can't render 3D/2D textured rectangle 
 * 3 Canvas2D can't directly upload font image.
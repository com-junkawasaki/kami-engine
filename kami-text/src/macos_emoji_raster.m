#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#import <stdbool.h>
#import <stdint.h>
#import <stdlib.h>
#import <string.h>

typedef struct KamiNativeEmojiRaster {
  uint8_t *rgba;
  uintptr_t len;
  uint32_t width;
  uint32_t height;
  float advance;
  float baseline_from_top;
} KamiNativeEmojiRaster;

bool kami_render_native_emoji_rgba(const char *cluster_utf8,
                                   float font_size,
                                   uint32_t canvas_size,
                                   KamiNativeEmojiRaster *out) {
  if (cluster_utf8 == NULL || out == NULL || canvas_size == 0) {
    return false;
  }

  @autoreleasepool {
    NSString *cluster = [NSString stringWithUTF8String:cluster_utf8];
    if (cluster == nil || cluster.length == 0) {
      return false;
    }

    NSInteger pixelsWide = (NSInteger)canvas_size;
    NSInteger pixelsHigh = (NSInteger)canvas_size;
    NSBitmapImageRep *rep = [[NSBitmapImageRep alloc]
        initWithBitmapDataPlanes:NULL
                      pixelsWide:pixelsWide
                      pixelsHigh:pixelsHigh
                   bitsPerSample:8
                 samplesPerPixel:4
                        hasAlpha:YES
                        isPlanar:NO
                  colorSpaceName:NSDeviceRGBColorSpace
                     bitmapFormat:NSBitmapFormatAlphaNonpremultiplied
                      bytesPerRow:0
                     bitsPerPixel:0];
    if (rep == nil) {
      return false;
    }

    NSGraphicsContext *ctx =
        [NSGraphicsContext graphicsContextWithBitmapImageRep:rep];
    if (ctx == nil) {
      return false;
    }

    [NSGraphicsContext saveGraphicsState];
    [NSGraphicsContext setCurrentContext:ctx];

    [[NSColor clearColor] setFill];
    NSRectFill(NSMakeRect(0, 0, canvas_size, canvas_size));

    NSMutableParagraphStyle *paragraph = [[NSMutableParagraphStyle alloc] init];
    paragraph.alignment = NSTextAlignmentCenter;

    NSFont *font = [NSFont fontWithName:@"Apple Color Emoji" size:font_size];
    if (font == nil) {
      font = [NSFont systemFontOfSize:font_size];
    }

    NSDictionary *attributes = @{
      NSFontAttributeName : font,
      NSParagraphStyleAttributeName : paragraph,
      NSForegroundColorAttributeName : [NSColor whiteColor],
    };

    CGFloat ascender = font.ascender;
    CGFloat descender = fabs(font.descender);
    CGFloat lineHeight = MAX(ascender + descender, font_size);
    CGFloat drawY = floor((canvas_size - lineHeight) * 0.5);
    NSRect drawRect = NSMakeRect(0, drawY, canvas_size, lineHeight);
    [cluster drawInRect:drawRect withAttributes:attributes];

    [ctx flushGraphics];
    [NSGraphicsContext restoreGraphicsState];

    unsigned char *bitmapData = [rep bitmapData];
    NSInteger bytesPerRow = [rep bytesPerRow];
    if (bitmapData == NULL || bytesPerRow <= 0) {
      return false;
    }

    uintptr_t len = (uintptr_t)(canvas_size * canvas_size * 4);
    uint8_t *copy = malloc(len);
    if (copy == NULL) {
      return false;
    }

    for (uint32_t y = 0; y < canvas_size; y++) {
      memcpy(copy + (uintptr_t)y * canvas_size * 4,
             bitmapData + (uintptr_t)y * bytesPerRow,
             (size_t)canvas_size * 4);
    }

    out->rgba = copy;
    out->len = len;
    out->width = canvas_size;
    out->height = canvas_size;
    out->advance = ceil([cluster sizeWithAttributes:attributes].width);
    out->baseline_from_top = drawY + ascender;
    return true;
  }
}

void kami_free_native_emoji_rgba(uint8_t *ptr, uintptr_t len) {
  (void)len;
  if (ptr != NULL) {
    free(ptr);
  }
}

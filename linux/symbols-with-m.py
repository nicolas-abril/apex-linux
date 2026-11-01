import sys
from fontTools.ttLib import TTFont
from fontTools.pens.ttGlyphPen import TTGlyphPen
src, dst = sys.argv[1], sys.argv[2]
f = TTFont(src)
cm = f.getBestCmap()
adv = f['hmtx'][cm[0xE0A0]][0]
# a blank 'm', so gpui's text system (cosmic-text), which drops a face with no 'm', keeps this one
order = f.getGlyphOrder() + ['m']
f['glyf'].glyphs['m'] = TTGlyphPen(None).glyph()
f.setGlyphOrder(order)
f['glyf'].glyphOrder = order
f['hmtx']['m'] = (adv, 0)
if 'post' in f and hasattr(f['post'], 'extraNames'):
    pass
f['maxp'].numGlyphs = len(order)
for t in f['cmap'].tables:
    if t.format in (4, 12) and t.isUnicode():
        t.cmap[ord('m')] = 'm'
f.save(dst)
g = TTFont(dst); print("after m:", g.getBestCmap().get(ord('m')), g['hmtx']['m'], adv, g['maxp'].numGlyphs)

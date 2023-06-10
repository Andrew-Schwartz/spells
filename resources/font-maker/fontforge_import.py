import sys

import fontforge

icons = sys.argv[1:]
svg_path = "/mnt/c/Users/andre/Downloads/bootstrap-icons-1.10.5/bootstrap-icons-1.10.5"

font = fontforge.open('Base.sfd')
for i, icon in enumerate(icons):
    uni = 0x61 + i
    # uni = 0xf102 + i
    font.createChar(uni, icon)
    glyph = font.createChar(uni)
    glyph.importOutlines(f'{svg_path}/{icon}.svg', 'correctdir')
    glyph.removeOverlap()

# font.save('Created.sfd')
font.generate('SpellsIcons.ttf')

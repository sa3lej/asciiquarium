#!/usr/bin/env python3
"""
Grand Underwater Castle Builder for Asciiquarium.

Uses a single-source grid. Each cell = (art_char, mask_char).
Fixed-width putl/putt/putr helpers enforce column alignment.

Target: 57 wide x 19 tall.
Layout: left_wing[0..19] | tower[20..37] | right_wing[38..56]

Color mask: R/r=red Y/y=yellow C/c=cyan M/m=magenta G/g=green B/b=blue W=white ' '=default
"""
import sys

W = 57
H = 20

LW = 20  # left wing cols 0-19
TW = 18  # tower cols 20-37
RW = 19  # right wing cols 38-56
assert LW + TW + RW == W


def make_grid():
    return [[(' ', ' ')] * W for _ in range(H)]


def put(grid, row, col, art_str, mask_str):
    assert len(art_str) == len(mask_str), \
        f"put({row},{col}): art={len(art_str)} mask={len(mask_str)}\n  a:|{art_str}|\n  m:|{mask_str}|"
    for i, (a, m) in enumerate(zip(art_str, mask_str)):
        c = col + i
        if 0 <= row < H and 0 <= c < W:
            grid[row][c] = (a, m)


def _fix(s, width):
    """Pad or truncate string to exact width."""
    return s.ljust(width)[:width]


def putl(g, row, art, mask):
    put(g, row, 0, _fix(art, LW), _fix(mask, LW))


def putt(g, row, art, mask):
    put(g, row, LW, _fix(art, TW), _fix(mask, TW))


def putr(g, row, art, mask):
    put(g, row, LW + TW, _fix(art, RW), _fix(mask, RW))


def grid_to_strings(grid):
    art_lines = []
    mask_lines = []
    for row in grid:
        a = ''.join(c[0] for c in row).rstrip()
        m = ''.join(c[1] for c in row).rstrip()
        art_lines.append(a)
        mask_lines.append(m)
    while art_lines and not art_lines[-1]:
        art_lines.pop()
        mask_lines.pop()
    # Pad art and mask lines to same length per row
    for i in range(len(art_lines)):
        la, lm = len(art_lines[i]), len(mask_lines[i])
        if la < lm:
            art_lines[i] = art_lines[i].ljust(lm)
        elif lm < la:
            mask_lines[i] = mask_lines[i].ljust(la)
    return art_lines, mask_lines


def build_frame(frame):
    g = make_grid()

    S  = 'M' if frame == 1 else 'm'
    WI = 'Y' if frame == 1 else 'G'
    ms = 'g' if frame == 1 else 'G'
    MS = 'G' if frame == 1 else 'g'
    GL = 'Y' if frame == 1 else 'G'

    # ── Rows 0-5: Turrets + tower peak (absolute placement) ──

    # Row 0: Pennants
    if frame == 1:
        put(g, 0, 3,  ">~]#[",  "RRRYR")
        put(g, 0, 27, "]#[~~>", GL+"RYRRR")
    else:
        put(g, 0, 4,  ">]#[",   "RRGR")
        put(g, 0, 27, "]#[~>",  GL+"RGRR")

    # Row 1: Flagpoles
    put(g, 1, 6,  "||", "RR")
    put(g, 1, 28, "||", "RR")

    # Row 2: Three turret caps
    put(g, 2, 5,  "/^^\\", "CccC")
    put(g, 2, 26, "/^^\\", "CccC")
    put(g, 2, 47, "/^^\\", "CccC")

    # Row 3
    put(g, 3, 5,  "|  |", "c  c")
    put(g, 3, 25, "/ ** \\", f"C {S}{S} C")
    put(g, 3, 47, "|  |", "c  c")

    # Row 4
    put(g, 4, 5,  "|  |", "c  c")
    put(g, 4, 24, "/ *{}* \\", f"C {S}{GL}{GL}{S} C")
    put(g, 4, 47, "|  |", "c  c")

    # Row 5
    put(g, 5, 5,  "|  |", "c  c")
    put(g, 5, 23, "/ * <||> * \\", f"C {S} {GL}RR{GL} {S} C")
    put(g, 5, 47, "|  |", "c  c")

    # ── Row 6: Wing crenellations + tower top ──
    putl(g, 6,
         " _   _   _   _   _ ",
         " c   c   c   c   c ")
    putt(g, 6,
         f"/ *            * \\",
         f"C {S}            {S} C")
    putr(g, 6,
         " _   _   _   _   _",
         " c   c   c   c   c")

    # ── Row 7: Full crenellation + rose window ──
    #                    12345678901234567890
    putl(g, 7,
         "[ ]_[ ]_[ ]_[ ]_[ ]_",
         "CcCcCcCcCcCcCcCcCcCc")
    #                    123456789012345678
    putt(g, 7,
         f"| *   <**>   * |",
         f"C {S}   R{S}{S}R   {S} C")
    #                    1234567890123456789
    putr(g, 7,
         "_[ ]_[ ]_[ ]_[ ]_[]",
         "cCcCcCcCcCcCcCcCcCC")

    # ── Body rows 8-18 ──

    # Row 8
    putl(g, 8,
         "|_=-_=_-_|=-_=_=-_=|",
         f"c{ms}c{MS}c{ms}c{ms}c{ms}c{ms}    c  c")
    putt(g, 8,
         " *              * ",
         f" {S}              {S} ")
    putr(g, 8,
         "|=_=-_=_-|=-_=_=-|",
         f"c  c    c{ms}c{MS}c{ms}c{ms}c{ms}c")

    # Row 9
    putl(g, 9,
         "| _- =    | =_=_=-|",
         "c         c       c")
    putt(g, 9,
         f" * *    **    * * ",
         f" {S} {S}    {S}{S}    {S} {S} ")
    putr(g, 9,
         "|-_=_=  | _- =   |",
         "c       c        c")

    # Row 10
    putl(g, 10,
         "|= -{ }   | -=_=-_|",
         f"c   {WI}{WI}    c       c")
    putt(g, 10,
         f"  * *  **  * *  ",
         f"  {S} {S}  {S}{S}  {S} {S}  ")
    putr(g, 10,
         "|-=_=-  | -={ }  |",
         f"c       c   {WI}{WI}   c")

    # Row 11
    putl(g, 11,
         f"| =_  {ms}   | =- =-_|",
         f"c   {ms} {ms}   c       c")
    putt(g, 11,
         f" * *  <||>  * * ",
         f" {S} {S}  {GL}RR{GL}  {S} {S} ")
    putr(g, 11,
         f"|=- -=  | =_  {ms}  |",
         f"c       c   {ms} {ms}  c")

    # Row 12
    putl(g, 12,
         "|= -{ }   | _=_=-_|",
         f"c   {WI}{WI}    c       c")
    putt(g, 12,
         f"  * *  **  * *  ",
         f"  {S} {S}  {S}{S}  {S} {S}  ")
    putr(g, 12,
         "|-=_=-  | _={ }  |",
         f"c       c   {WI}{WI}   c")

    # Row 13
    putl(g, 13,
         f"| =_  {MS}   | =- =-_|",
         f"c   {MS} {MS}   c       c")
    putt(g, 13,
         f" *              * ",
         f" {S}              {S} ")
    putr(g, 13,
         f"|=- -=  | =_  {MS}  |",
         f"c       c   {MS} {MS}  c")

    # Row 14
    putl(g, 14,
         "|= -{ }   | _=_=-_|",
         f"c   {WI}{WI}    c       c")
    putt(g, 14,
         "-    /^^^^\\    -",
         "      CccccC      ")
    putr(g, 14,
         "|-=_=-  | -={ }  |",
         f"c       c   {WI}{WI}   c")

    # Row 15
    putl(g, 15,
         "| =_      | =- =-_|",
         "c         c       c")
    putt(g, 15,
         f" = * |####| * = ",
         f"   {GL} yrrry {GL}    ")
    putr(g, 15,
         "|=- -=  | =_     |",
         "c       c        c")

    # Row 16
    putl(g, 16,
         "| = -     | _=_=-_|",
         "c         c       c")
    putt(g, 16,
         f" = * |#  #| * = ",
         f"   {GL} yr  ry {GL}    ")
    putr(g, 16,
         "|-=_=-  | =-     |",
         "c       c        c")

    # Row 17
    putl(g, 17,
         "|- =_     | =- =-_|",
         "c         c       c")
    putt(g, 17,
         " =   |#  #|   = ",
         "     yr  ry       ")
    putr(g, 17,
         "|=- -=  |- =_    |",
         "c       c        c")

    # Row 18
    putl(g, 18,
         "|_________|_______|",
         "ccccccccccccccccccc")
    putt(g, 18,
         f"______|    |______",
         f"ccccc{GL}    {GL}cccccc")
    putr(g, 18,
         "|_______|_________|",
         "ccccccccccccccccccc")

    return grid_to_strings(g)


def verify(art_lines, mask_lines, label):
    ok = True
    if len(art_lines) != len(mask_lines):
        print(f"ERROR [{label}]: line count mismatch")
        ok = False
    for i, (a, m) in enumerate(zip(art_lines, mask_lines)):
        if len(a) != len(m):
            print(f"  MISMATCH [{label}] line {i}: art={len(a)} mask={len(m)}")
            print(f"    art:  |{a}|")
            print(f"    mask: |{m}|")
            ok = False
    return ok


def main():
    all_ok = True
    results = {}

    for fn in [1, 2]:
        art_lines, mask_lines = build_frame(fn)
        max_w = max(max((len(l) for l in art_lines), default=0),
                    max((len(l) for l in mask_lines), default=0))

        print(f"\n{'='*60}")
        print(f"FRAME {fn}  ({max_w} x {len(art_lines)})")
        print(f"{'='*60}")

        ok = verify(art_lines, mask_lines, f"Frame {fn}")
        all_ok = all_ok and ok
        if ok:
            print("  Alignment: ALL OK")

        print("\nART:")
        for i, line in enumerate(art_lines):
            print(f"  {i:2d} |{line}|")
        print("\nMASK:")
        for i, line in enumerate(mask_lines):
            print(f"  {i:2d} |{line}|")

        results[fn] = (art_lines, mask_lines)

    if not all_ok:
        print("\n*** FIX ALIGNMENT ERRORS ***")
        sys.exit(1)

    # Rust output
    print("\n" + "=" * 60)
    print("RUST CODE")
    print("=" * 60)
    for fn in [1, 2]:
        art_lines, mask_lines = results[fn]
        target = max(max(len(l) for l in art_lines), max(len(l) for l in mask_lines))
        art_p = [l.ljust(target) for l in art_lines]
        mask_p = [l.ljust(target) for l in mask_lines]
        art_e = '\n'.join(art_p).replace('\\', '\\\\')
        mask_e = '\n'.join(mask_p).replace('\\', '\\\\')

        desc = ("flag streams right, bright magenta glass, yellow windows"
                if fn == 1 else
                "flag flutters, dark magenta glass, green windows")
        print(f'\n// Frame {fn}: {desc}')
        print(f'const CASTLE_ART_{fn}: &str = "\\')
        for line in art_e.split('\n'):
            print(line)
        print('";')
        print(f'\nconst CASTLE_MASK_{fn}: &str = "\\')
        for line in mask_e.split('\n'):
            print(line)
        print('";')


if __name__ == '__main__':
    main()

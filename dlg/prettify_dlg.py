import os.path
import re
import sys
import logging

logging.basicConfig(level=logging.DEBUG)

def main():
    file = sys.argv[1]
    root, _ = os.path.splitext(file)

    pattern = re.compile(r"^\{(\d+)}\{(.+)}\{(.*)}\{(\d*)}\{(.*)}\{(-?\d*)}\{(.*)}")

    line_groups: list[tuple[str,...]] = []
    group_widths: list[int] = [0,0,0,0,0,0,0]

    if not os.path.isfile(file):
        print(f"File {file!r} not found.")
        return

    with (open(file, "r", encoding="utf-8") as fp):
        for line in fp.read().splitlines():
            if not line:
                continue
            logging.debug(line)
            if match := pattern.match(line):
                groups = match.groups()

                for i in range(7):
                    group_widths[i] = max(group_widths[i], len(groups[i]))

                line_groups.append(match.groups())
                logging.info("Successfully parsed line")
            else:
                logging.warning("Could not parse line")

    s = ""
    for group in line_groups:
        s += "{ "
        s += " }{ ".join(g.rjust(w) if g.isdigit() else g.ljust(w) for g, w in zip(group, group_widths))
        s += " }\n"

    file = root + "_pretty.dlg"
    with open(file, "w", encoding="utf-8") as fp:
        fp.write(s)




if __name__ == "__main__":
    main()
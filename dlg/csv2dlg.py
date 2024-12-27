import os.path
import re
import sys


def main():
    file = sys.argv[1]
    root, _ = os.path.splitext(file)

    pattern = re.compile(r"^\{(\d+)}\{(.+)}\{(.*)}\{(\d*)}\{(\w*)}\{(\d*)}\{(\w*)}")

    line_groups: list[tuple[str,...]] = []
    group_widths: list[int] = [0,0,0,0,0,0,0]

    if not os.path.isfile(file):
        print(f"File {file!r} not found.")
        return

    with open(file, "r", encoding="utf-8") as fp:
        for line in fp:
            if match := pattern.match(line):
                groups = match.groups()

                for i in range(7):
                    group_widths[i] = max(group_widths[i], len(groups[i]))

                line_groups.append(match.groups())

    s = ""
    for group in line_groups:
        s += "{ "
        s += " }{ ".join(g.rjust(w) if g.isdigit() else g.ljust(w) for g, w in zip(group, group_widths))
        s += " }\n"

    file = root + ".dlg"
    with open(file, "w", encoding="utf-8") as fp:
        fp.write(s)




if __name__ == "__main__":
    main()
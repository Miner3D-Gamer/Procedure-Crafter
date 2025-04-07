NATIVE = True


import os, time

this = os.path.dirname(os.path.realpath(__file__))
if NATIVE:
    command_file = os.path.join(this, "native_run.txt")
else:
    command_file = os.path.join(this, "web_run.txt")

with open(command_file, "r") as f:
    c = f.read().split("\n")
    c = [
        line[: line.find("#")] if line.find("#") != -1 else line
        for line in c
        if len(line) > 0
    ]
for cmd in c:
    print("> %s" % cmd)
    try:
        os.system(cmd)
    except:
        pass
    time.sleep(0.01)

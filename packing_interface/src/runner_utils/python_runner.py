import inspect
import subprocess
import sys
import os
import json

result = subprocess.run(["cat", sys.argv[1]], capture_output=True, text=True, check=True)
dir_path = os.path.dirname(os.path.realpath(__file__))
lib_path = os.path.join(dir_path, '..', 'runner_lib')
sys.path.insert(0, lib_path)
import packing_lib

ns = {"__name__": "__main__", "make_output": packing_lib.make_output}
exec(result.stdout, ns)

packing_class = ns.get("Packing")
repacking_class = ns.get("Repacking")

if packing_class is not None and inspect.isclass(packing_class):
    s = getattr(packing_class, "solve", None)
    if s is None or not callable(s):
        raise TypeError("Class 'Packing' must define a callable method 'solve'")

    sig = inspect.signature(s)
    params = list(sig.parameters.values())
    if len(params) != 3:
        raise TypeError("Packing.solve must accept 3 params (self, bin_width, rectangles)")

    bin_width = int(sys.argv[2])
    rectangles = json.loads(sys.argv[3])

    instance = packing_class()
    out = instance.solve(bin_width, rectangles)
    print(out)

elif repacking_class is not None and inspect.isclass(repacking_class):
    s = getattr(repacking_class, "solve", None)
    if s is None or not callable(s):
        raise TypeError("Class 'Repacking' must define a callable method 'solve'")
    sig = inspect.signature(s)
    params = list(sig.parameters.values())
    if len(params) != 5:
        raise TypeError("Packing.solve must accept 3 params (self, bin_width, rectangles)")

    bin_height = int(sys.argv[2])
    bin_width = int(sys.argv[3])
    rectangles = json.loads(sys.argv[4])
    non_empty_space = json.loads(sys.argv[5])

    print(f"[repack] bin_height={bin_height} bin_width={bin_width} rects={len(rectangles)} obstacles={len(non_empty_space)}", file=sys.stderr)
    print(f"[repack] non_empty_space={non_empty_space}", file=sys.stderr)

    instance = repacking_class()
    out = instance.solve(bin_height, bin_width, rectangles, non_empty_space)
    print(out)

else:
    raise TypeError("Algorithm must define class named 'Packing' or 'Repacking'")

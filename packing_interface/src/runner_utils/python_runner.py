import inspect
import subprocess
import sys
import os 

result = subprocess.run(["cat", sys.argv[1]], capture_output=True,text=True,check=True)
dir_path = os.path.dirname(os.path.realpath(__file__))
lib_path = os.path.join(dir_path, '..', 'runner_lib')
sys.path.insert(0, lib_path)
import packing_lib

ns = {"__name__": "__main__", "make_output": packing_lib.make_output}
#ns = {"__name__": "__main__"}
exec(result.stdout, ns)

p = ns.get("Packing")
if p is None or not inspect.isclass(p):
    raise TypeError("Algorithm must define class named Packing")

s = getattr(p, "solve", None)
if s is None or not callable(s):
    raise TypeError("Class 'Packing' must define a callable method 'solve'")

sig = inspect.signature(s)
params = list(sig.parameters.values())
if len(params) != 3:
    raise TypeError("Packing must accept 3 params")

instance = p()
out = instance.solve(1, [1])
print(out)

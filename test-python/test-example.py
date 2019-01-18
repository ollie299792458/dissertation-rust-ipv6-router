import subprocess

subprocess.run(["mn", "--custom", "topo-example.py", "--topo", "mytopo",\
        "--test", "pingall"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,\
        stderr=subprocess.PIPE)

import subprocess
import sys

subprocess.call(['./jack_compiler', '--path', sys.argv[1]])

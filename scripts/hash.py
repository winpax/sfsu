import sys
import hashlib

# Totally arbitrary
BUF_SIZE = 65536  # 64 kb

sha1 = hashlib.sha256()

with open(sys.argv[1], "rb") as f:
    while True:
        data = f.read(BUF_SIZE)
        if not data:
            break

        sha1.update(data)

print(sha1.hexdigest(), end="")

import sys
import hashlib

# Totally arbitrary
BUF_SIZE = 65536  # 64 kb
TO_HASH = sys.argv[1]

sha1 = hashlib.sha256()

with open(TO_HASH, "rb") as f:
    while True:
        data = f.read(BUF_SIZE)
        if not data:
            break

        sha1.update(data)


with open(TO_HASH + ".sha256", "w", encoding="utf8") as f:
    f.write(sha1.hexdigest())

import json
import sys

lines = sys.stdin.read().strip().splitlines()

nums = json.loads(lines[0])
target = int(lines[1])

seen = {}

for i, num in enumerate(nums):
  complement = target - num

  if complement in seen:
    print(json.dumps([seen[complement], i]))
    break

  seen[num] = i

# class Solution:
#     def reverseDegree(self, s: str) -> int:
#         lowercase_str = string.ascii_lowercase
#         reversed = list(lowercase_str)[::-1]
#         total = 0
#         for i, c in enumerate(s):
#           contribution = (reversed.index(c)+1) * (i+1)
#           print(contribution)
#           total += contribution
#         return total

# class Solution:
#   def reverseDegree(self, s: str) -> int:
#     total = 0
#     values = {
#         c: 26 - (ord(c) - ord('a'))
#         for c in string.ascii_lowercase
#     }
#     for i, c in enumerate(s, 1):
#         total += values[c] * i
#     return total
# class Solution:
#     def reverseDegree(self, s: str) -> int:
#         total = 0
#         for i, c in enumerate(s, 1):
#             total += (123 - ord(c)) * i
#         return total
class Solution:
    def reverseDegree(self, s: str) -> int:
        total = 0
        for i, c in enumerate(s.encode(), 1):
            total += (123 - c) * i
        return total
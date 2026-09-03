class Solution:
  def uniformArray(self, A: list[int]) -> bool:
    return not (min(A) ^ reduce(or_, A)) & 1


# class Solution:
#   def uniformArray(self, nums1: list[int]) -> bool:
#     return (
#       # Check whether there are odd elements and get min of these
#       (modd := min((n for n in nums1 if n & 1 == 1), default=0)) == 0
#       or
#       # Check whether there are even elements and whether we
#       # may substract `modd` from each of these
#       modd < min((n for n in nums1 if n & 1 == 0), default=1e9 + 1)
#     )

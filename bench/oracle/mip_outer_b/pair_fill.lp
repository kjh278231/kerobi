Minimize
 obj: p1 + p2 + n1 + n2 + z
Subject To
 pair: p1 + 2 p2 - n1 - 2 n2 = 0
 cap: p1 + p2 + n1 + n2 + z <= 40
 solo: p1 + p2 + z <= 50
Bounds
 0 <= p1 <= 8
 0 <= p2 <= 7
 0 <= n1 <= 6
 0 <= n2 <= 5
Binary
 z
End

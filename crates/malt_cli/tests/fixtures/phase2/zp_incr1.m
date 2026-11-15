function zp_incr1()
s = 0
parfor i = 1:10
s = max(s, i)
s = min(s, i)
end
end

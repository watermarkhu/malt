function zp_save6(f)
s = struct('a', 0)
parfor i = 1:10
s(i).a = i
end
save(f, 's')
end

function zp_save10(f)
a = zeros(1, 10)
parfor i = 1:10
a(i) = i
end
save(f, 'a')
disp(a)
end

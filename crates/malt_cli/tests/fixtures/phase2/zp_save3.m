function zp_save3(f)
a = zeros(1, 10)
parfor i = 1:10
a(i) = i
end
save(f, 'a')
end

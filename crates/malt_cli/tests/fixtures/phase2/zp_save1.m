function zp_save1(f)
x = 0
parfor i = 1:10
x = i
end
save(f, 'x')
end

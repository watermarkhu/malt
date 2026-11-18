function f()
parfor k = 1:3, q = k; end
save('q.mat', 'q')
end

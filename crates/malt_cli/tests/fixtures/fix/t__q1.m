a = [1 2 3]; s.x = {1,2}; c = {[1 2]};
parfor i=1:3
  if i>1, break, end
end
v1 = s.x{end};
v2 = c{1}(end);
v3 = a(a(end));
v4 = s.x(end);
v5 = a(end).y;
v6 = numel(a(1:end));
v7 = a(:, end);
v8 = max(a(end), 1);
fh = @(x) x(end);
v9 = a(end-1:end);
if a(end) == 3, disp(1), end
for k = 1:a(end), end
spmd
  q = 1;
end

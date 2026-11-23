function r = legacy2(s)
% String handling, old-style
r = '';
if strcmp(s, 'a') | strcmp(s, 'b')
    r = [r 'ab'];
elseif ~isempty(findstr(s, 'c'))
    r = [r 'c'];
end
n = length(s);
for i = 1:n
    c = s(i);
    switch c
        case 'x'
            r = [r upper(c)];
        case {'y', 'z'}
            r = [r lower(c)];
    end
end
fid = fopen('out.txt', 'w');
fprintf(fid, '%s\n', r);
fclose(fid);
A = ones(3);
B = A(:, end);
t = cputime;
r = strcat(r, num2str(t));

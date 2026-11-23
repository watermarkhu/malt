function h_ctpct_message(x, y)
% The first argument is the format unless it is an identifier or a control word.
if x > 1
    error('msg', y);
end
if x > 2
    warning('msg', y);
end
if x > 3
    error('A:b', 'fmt', 1);
end
if x > 4
    error('A:b', 'fmt %d', 1);
end
if x > 5
    warning('off', 'all');
end
if x > 6
    error('msg %d', 1);
end
if x > 7
    warning('on', 'MATLAB:something');
end
end

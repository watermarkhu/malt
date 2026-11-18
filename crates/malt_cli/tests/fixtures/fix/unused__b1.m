function b1()
    persistent cache
    if isempty(cache), cache = 1; end
    global G
    G = 5;
    q(2) = 1;
    s.a = 1;
    w = 1;
    who
    k = 3;
    evalin('caller','k');
end

function out = strempneg(s, t)
%STREMPNEG The negated and plain strfind containment forms.
a = ~isempty(strfind(s, 'needle'));
b = isempty(strfind(t,'x'));
if a && b
    out = 1;
else
    out = 0;
end
end

function secondpass__msnu(a)
%SECONDPASS__MSNU Directives that suppress nothing any more (MSNU).
disp(a); %#ok<AGROW>
y = 2; %#ok<NASGU,AGROW>
z = 3; %#ok<AGROW,NASGU>
disp(a); %#ok<AGROW> remark
disp(1); %#ok
q = 5; %#ok<NASGU>
end

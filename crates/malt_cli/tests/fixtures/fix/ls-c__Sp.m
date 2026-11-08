parfor i=1:3
  save foo -fromstruct s
  save('foo','-fromstruct','s')
  y = nargin;
end

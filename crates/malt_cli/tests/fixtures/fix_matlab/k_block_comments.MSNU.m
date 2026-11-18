% K_BLOCK_COMMENTS
% help text placed BEFORE the function line (sometimes people do this)
% second help line, still before the function line
function out = k_block_comments(in)
%K_BLOCK_COMMENTS help after the function line
%   out = k_block_comments(in) does a small amount of work.
%   The old version is kept below in a block comment.

%%  Section one: setup
out = in;

%{
old = in * 3;
for q = 1:10
  old = old + q;
end
%}

%% Section two  TODO: check the edge cases
%{
Old implementation kept for reference:
  tmp = zeros(size(in));
  tmp(:) = in(:) .^ 2;
  out = tmp;
%}

% out = out.^2;   % disabled, was too aggressive
% FIXME: ??? wrong when in is empty
if isempty(in) % TODO handle empty input
    out = 0;
end

%% Section three: the real work
x = 1; % x = 2;
% disp('debug');
% fprintf('value %g\n', x);

%{
   Another old note, multi line.
   Nothing here runs.
   x = x + 100;
%}

out = out + x;  

%% Section four: TODO remove the next line once in is validated
% out = out * 1000;
if ~isnumeric(out)  % TODO: use validateattributes instead
    error('k:type', 'numeric input expected');
end

%{ commented-out loop, inline opener is a plain comment
for r = 1:3
    out = out + r;
end
%}

% NOTE: the block below was copied from another file
%   a = b';
%   c = a(1, :);
%   d = c.^2;  % squared

%% Section five: wrap up
% XXX this is a placeholder for the reporting step
out = double(out);   % make sure the type is right
% HACK: scale by 1 to keep old callers happy
out = out * 1;

end

% the trailing comment block below sits after the function
% it was used by an older helper and is not called anymore
%{
function legacy = old_helper(v)
legacy = v;
end
%}

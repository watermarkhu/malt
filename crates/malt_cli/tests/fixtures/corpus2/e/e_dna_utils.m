function varargout = e_dna_utils(op, seq, varargin)
%E_DNA_UTILS Small helpers for working with DNA sequences.
%   rc = e_dna_utils('revcomp', seq) returns the reverse complement.
%   gc = e_dna_utils('gc', seq) returns the GC fraction in [0, 1].
%   [kmers, counts] = e_dna_utils('kmers', seq, k) counts k-mers.
%
%   Only the bases A, C, G, T (and N) are accepted, case-insensitive.

if nargin < 2
    error('e_dna_utils:notEnoughInputs', 'Need an operation and a sequence.');
end

seq = upper(char(seq));
validateSequence(seq);

switch lower(op)
    case 'revcomp'
        varargout{1} = reverseComplement(seq);
    case 'gc'
        varargout{1} = gcContent(seq);
    case 'kmers'
        if isempty(varargin)
            k = 3;
        else
            k = varargin{1};
        end
        [varargout{1}, varargout{2}] = countKmers(seq, k);
    otherwise
        error('e_dna_utils:unknownOp', 'Unknown operation "%s".', op);
end
end

function rc = reverseComplement(seq)
% Lookup-table approach: map every base to its complement.
persistent compMap
if isempty(compMap)
    compMap = containers.Map({'A', 'C', 'G', 'T', 'N'}, {'T', 'G', 'C', 'A', 'N'});
end

comp = cell(1, numel(seq));
for i = 1:numel(seq)
    comp{i} = compMap(seq(i));
end
rc = fliplr([comp{:}]);
end

function gc = gcContent(seq)
% Fraction of G and C bases, ignoring N.
isGC = seq == 'G' | seq == 'C';
isN = seq == 'N';
nonN = numel(seq) - sum(isN);
if nonN == 0
    gc = NaN;
else
    gc = sum(isGC) / nonN;
end
end

function [kmers, counts] = countKmers(seq, k)
% Count all overlapping k-mers of length k.
if k > numel(seq)
    kmers = {};
    counts = [];
    return
end

nKmers = numel(seq) - k + 1;
codes = zeros(1, nKmers);
baseIdx = containers.Map({'A', 'C', 'G', 'T'}, {0, 1, 2, 3});
baseVals = zeros(1, numel(seq));
for i = 1:numel(seq)
    if isKey(baseIdx, seq(i))
        baseVals(i) = baseIdx(seq(i));
    else
        baseVals(i) = -1;
    end
end

% Encode each window as a base-4 integer.
for j = 1:nKmers
    window = baseVals(j:j+k-1);
    if any(window < 0)
        codes(j) = -1;
    else
        codes(j) = sum(window .* 4.^(k-1:-1:0));
    end
end

codes = codes(codes >= 0);
[uniqCodes, ~, idx] = unique(codes);
counts = histcounts(idx, 1:numel(uniqCodes)+1);

kmers = cell(size(uniqCodes));
alphabet = 'ACGT';
for u = 1:numel(uniqCodes)
    code = uniqCodes(u);
    digits = zeros(1, k);
    for d = k:-1:1
        digits(d) = mod(code, 4);
        code = floor(code / 4);
    end
    kmers{u} = alphabet(digits + 1);
end
end

function validateSequence(seq)
% Reject anything that is not a nucleotide code.
if ~all(ismember(seq, 'ACGTN'))
    bad = unique(seq(~ismember(seq, 'ACGTN')));
    error('e_dna_utils:badBase', 'Invalid bases found: %s', bad);
end
end

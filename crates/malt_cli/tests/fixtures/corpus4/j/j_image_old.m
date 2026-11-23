function out = j_image_old(fname, thresh)
%J_IMAGE_OLD  Simple image processing for the vision project, 2001 style.
%   OUT = J_IMAGE_OLD(FNAME) reads an image, converts it to grey, thresholds
%   it and returns the binary mask.  OUT = J_IMAGE_OLD(FNAME, THRESH) sets
%   the threshold (0..255).

if nargin < 2
    thresh = 128;
end

I = imread(fname);
if ndims(I) == 3
    G = rgb2gray(I);
else
    G = I;
end
G = double(G);
[nr, nc] = size(G);
fprintf('image is %d by %d\n', nr, nc)

% threshold with explicit loops, the way the course taught it
B = zeros(nr, nc);
for r = 1:nr
    for c = 1:nc
        if G(r, c) > thresh
            B(r, c) = 255;
        end
    end
end

% median filter by hand, 3 by 3 window
M = G;
for r = 2:nr-1
    for c = 2:nc-1
        win = G(r-1:r+1, c-1:c+1);
        w = sort(win(:));
        M(r, c) = w(5);
    end
end

% histogram of the grey levels, built with a loop
hist_counts = zeros(1, 256);
for r = 1:nr
    for c = 1:nc
        v = round(G(r, c)) + 1;
        if v < 1, v = 1; end
        if v > 256, v = 256; end
        hist_counts(v) = hist_counts(v) + 1;
    end
end

% show the results in figure 1
figure(1)
clf
subplot(1, 3, 1), imshow(uint8(G)), title('grey')
subplot(1, 3, 2), imshow(uint8(B)), title('threshold')
subplot(1, 3, 3), imshow(uint8(M)), title('median')

% colormap on an indexed image, the old way
[X, map] = gray2ind(uint8(G), 64);
figure(2)
imshow(X, map)
colormap(jet(64))

% scale the image to [0, 1] and back, with the old conversions
S = im2uint8(mat2gray(G));
D = im2double(S);
back = uint8(D * 255);
if isequal(back, uint8(G))
    disp('round trip exact')
else
    disp('round trip lost some levels')
end

% edge image with the sobel operator, computed by hand
kx = [-1 0 1; -2 0 2; -1 0 1];
ky = kx';
Gx = conv2(G, kx, 'same');
Gy = conv2(G, ky, 'same');
E = sqrt(Gx.^2 + Gy.^2);
E = E / max(E(:)) * 255;

% the number of object pixels per row, using find in a loop
rowcount = zeros(nr, 1);
for r = 1:nr
    idx = find(B(r, :) == 255);
    rowcount(r) = length(idx);
end
[maxrow, where] = max(rowcount);

% the result is returned as a struct with the main fields
out.mask = B;
out.median = M;
out.edges = uint8(E);
out.hist = hist_counts;
out.peakrow = where;
out.peakcount = maxrow;

% optional saving of the output as a bitmap
if exist('save_out.bmp', 'file')
    delete('save_out.bmp')
end
imwrite(uint8(B), 'save_out.bmp', 'bmp');

% an RGB overlay when the input was colour
if ndims(I) == 3
    R = I;
    R(:, :, 1) = uint8(B);
    figure(3)
    image(R)
    axis image
    axis off
end

% a simple name for the output file, from the input name
[pathname, name, ext] = fileparts(fname);
outname = [name '_mask.mat'];
save(outname, 'B')
disp(['wrote ' outname])

% image sizes in the batch log
sz = size(I);
fprintf('channels %d\n', sz(3:end))

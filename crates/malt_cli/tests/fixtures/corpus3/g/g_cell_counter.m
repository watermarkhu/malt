function [count, labels, areas] = g_cell_counter(img, thr)
% G_CELL_COUNTER count bright blobs in a grayscale image by hand
%   n = g_cell_counter(imread('cells.tif'), 120)
% flood fill is done with a stack, not bwlabel (the old lab machines
% did not have the image toolbox)

if nargin < 2
    thr = 128;
end
if size(img,3) == 3
    img = rgb2gray(img);
end
img = double(img);
[nr, nc] = size(img);
bw = zeros(nr, nc);
for r = 1:nr
    for c = 1:nc
        if img(r,c) > thr
            bw(r,c) = 1;
        else
            bw(r,c) = 0;
        end
    end
end

% show the thresholded image
figure(1)
imshow(bw)
title('threshold')

labels = zeros(nr, nc);
count = 0;
areas = [];
for r = 1:nr
    for c = 1:nc
        if bw(r,c) == 1 && labels(r,c) == 0
            count = count + 1;
            stack = [r c];
            labels(r,c) = count;
            a = 0;
            while ~isempty(stack)
                p = stack(end,:);
                stack(end,:) = [];
                a = a + 1;
                rr = p(1); cc = p(2);
                for dr = -1:1
                    for dc = -1:1
                        r2 = rr + dr; c2 = cc + dc;
                        if r2 >= 1 && r2 <= nr && c2 >= 1 && c2 <= nc
                            if bw(r2,c2) == 1 && labels(r2,c2) == 0
                                labels(r2,c2) = count;
                                stack = [stack; r2 c2];
                            end
                        end
                    end
                end
            end
            areas(count) = a;
        end
    end
end
fprintf('found %d blobs\n', count);

% remove the tiny ones (noise), magic numbers all over
keep = 0;
for k = 1:count
    if areas(k) < 15
        labels(labels == k) = 0;
    elseif areas(k) > 900 | areas(k) < 0
        labels(labels == k) = 0;
    else
        keep = keep + 1;
    end
end
disp(['after cleaning: ' num2str(keep)])

% mean area, done with a loop because someone wanted it
tot = 0;
for k = 1:length(areas)
    tot = tot + areas(k);
end
meanArea = tot / length(areas)

% colour picture of the labels
figure(2)
imagesc(labels)
colormap jet
colorbar
axis image

% old: show the raw image in grey
% figure(3); imshow(uint8(img))

figure(3)
subplot(1,2,1)
imshow(uint8(img))
subplot(1,2,2)
imshow(uint8(bw*255))

% count rows and columns of the labelled image, unused
[lr, lc] = size(labels);
total_px = lr*lc;
ratio = sum(sum(labels > 0)) / total_px;
if ratio < 0.01
    disp('almost nothing found, check thr')
end

% duplicated again for the second threshold
thr2 = thr + 20;
bw2 = img > thr2;
n2 = sum(bw2(:));
fprintf('pixels above %d: %d\n', thr2, n2);
i = 0;
while i < 3
    i = i + 1;
end
end

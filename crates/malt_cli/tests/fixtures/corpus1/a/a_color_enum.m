classdef a_color_enum
    %A_COLOR_ENUM Plot colours used by the reporting tools.
    %   Each member carries an RGB triple and a short name.

    properties
        RGB (1,3) double
        ShortName (1,:) char
    end

    enumeration
        Red    ([0.85 0.10 0.10], 'red')
        Green  ([0.10 0.65 0.20], 'grn')
        Blue   ([0.10 0.30 0.85], 'blu')
        Grey   ([0.50 0.50 0.50], 'gry')
    end

    methods
        function obj = a_color_enum(rgb, name)
            obj.RGB = rgb;
            obj.ShortName = name;
        end

        function hex = toHex(obj)
            %TOHEX Return the colour as a '#RRGGBB' string.
            c = round(obj.RGB * 255);
            hex = sprintf('#%02X%02X%02X', c(1), c(2), c(3));
        end

        function tf = isDark(obj)
            %ISDARK True if the perceived luminance is below 0.5.
            lum = 0.299*obj.RGB(1) + 0.587*obj.RGB(2) + 0.114*obj.RGB(3);
            tf = lum < 0.5
        end
    end

    methods (Static)
        function c = fromName(name)
            %FROMNAME Look up a member by its short name (case-insensitive).
            members = enumeration('a_color_enum');
            idx = find(strcmpi({members.ShortName}, name), 1);
            if isempty(idx)
                error('a_color_enum:unknown', 'Unknown colour "%s".', name)
            end
            c = members(idx);
        end
    end
end

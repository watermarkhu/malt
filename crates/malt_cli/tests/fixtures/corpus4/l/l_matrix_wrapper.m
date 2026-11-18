classdef l_matrix_wrapper
    %L_MATRIX_WRAPPER Value wrapper around a numeric matrix with metadata.
    %   The wrapped data is copied on assignment like any MATLAB value.
    %   Rows and Cols are derived from Data; Checksum is kept for
    %   internal comparisons and is not shown in the default display.

    properties
        Data (:,:) double = zeros(0,0)
        Label (1,1) string = ""
    end

    properties (Dependent)
        Rows (1,1) double
        Cols (1,1) double
    end

    properties (Hidden)
        Checksum (1,1) double
    end

    properties (Access = private)
        CacheStamp
    end

    methods
        function obj = l_matrix_wrapper(data, label)
            %L_MATRIX_WRAPPER Wrap data with an optional label.
            arguments
                data (:,:) double = zeros(0,0)
                label (1,1) string = "unnamed"
            end
            obj.Data = data;
            obj.Label = label;
            obj.Checksum = sum(data(:));
        end

        function r = get.Rows(obj)
            r = size(obj.Data, 1);
        end

        function r = get.Cols(obj)
            r = size(obj.Data, 2);
        end

        function obj = set.Label(obj, value)
            obj.Label = strtrim(value);
        end

        function out = scaled(obj, factor)
            %SCALED Return a copy with Data multiplied by factor.
            arguments
                obj
                factor (1,1) double {mustBeFinite} = 1
            end
            out = obj;
            out.Data = obj.Data * factor;
        end

        function out = rescaled(obj)
            %RESCALED Return a copy with Data mapped to [0, 1].
            out = obj;
            out.Data = l_helpers1(obj.Data);
        end

        function s = saveobj(obj)
            %SAVEOBJ Plain struct used when the object is saved to a file.
            s = struct('Data', obj.Data, 'Label', char(obj.Label));
        end
    end

    methods (Static)
        function obj = loadobj(s)
            %LOADOBJ Rebuild the wrapper from a saved struct or object.
            if isstruct(s)
                obj = l_matrix_wrapper(s.Data, string(s.Label));
            else
                obj = s;
            end
        end

        function obj = fromRange(first, last, varargin)
            %FROMRANGE Wrapper around a row vector first:last.
            p = inputParser;
            addParameter(p, 'Label', "range");
            parse(p, varargin{:});
            obj = l_matrix_wrapper((first:last).', p.Results.Label);
        end
    end
end

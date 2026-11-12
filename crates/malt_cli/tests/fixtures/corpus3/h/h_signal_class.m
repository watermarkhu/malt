classdef h_signal_class
    %H_SIGNAL_CLASS Simple container for a sampled signal
    %   Value class: every operation returns a modified copy.
    %
    %   s = h_signal_class(data, fs)
    %   s = h_signal_class(data, fs, 'label', 'ch1', 'units', 'mV')

    properties
        Data (:,1) double = []
        Fs (1,1) double {mustBePositive} = 1
        Label string = ""
        Units char = 'V'
        Meta struct = struct()   % free-form, nobody reads this yet
    end

    properties (Dependent)
        Duration
        NSamples
    end

    properties (Constant)
        DEFAULT_FS = 1000;
    end

    methods
        function obj = h_signal_class(data, fs, varargin)
            % constructor, overloaded via nargin
            if nargin == 0
                return
            end
            if nargin >= 1
                obj.Data = data(:);
            end
            if nargin >= 2
                obj.Fs = fs;
            else
                obj.Fs = obj.DEFAULT_FS;
            end
            for k = 1:2:numel(varargin)
                switch lower(varargin{k})
                    case 'label'
                        obj.Label = varargin{k+1};
                    case 'units'
                        obj.Units = varargin{k+1};
                    otherwise
                        error('h_signal_class:badOption', 'Unknown option "%s"', varargin{k});
                end
            end
        end

        function d = get.Duration(obj)
            d = numel(obj.Data) / obj.Fs;
        end

        function n = get.NSamples(obj)
            n = numel(obj.Data);
        end

        function obj = set.Label(obj, v)
            obj.Label = string(v);
        end

        function r = plus(a, b)
            if isa(a, 'h_signal_class') && isa(b, 'h_signal_class')
                assert(a.Fs == b.Fs, 'Sample rates differ');
                n = min(numel(a.Data), numel(b.Data));
                r = h_signal_class(a.Data(1:n) + b.Data(1:n), a.Fs, 'label', a.Label + "+" + b.Label);
            elseif isa(a, 'h_signal_class')
                r = h_signal_class(a.Data + b, a.Fs, 'label', a.Label);
            else
                r = h_signal_class(a + b.Data, b.Fs, 'label', b.Label);
            end
        end

        function r = minus(a, b)
            r = a + (-1 * b);
        end

        function r = mtimes(a, k)
            if isa(a, 'h_signal_class')
                s = a;
                scale = k;
            else
                s = k;
                scale = a;
            end
            r = h_signal_class(s.Data * scale, s.Fs, 'label', s.Label);
        end

        function obj = detrendCopy(obj)
            obj.Data = obj.Data - mean(obj.Data);
        end

        function obj = resampleTo(obj, fsNew)
            % TODO: use resample() instead of interp1, interp1 has no anti-alias
            t = (0:numel(obj.Data)-1).' / obj.Fs;
            tNew = (0:floor(obj.Duration*fsNew)-1).' / fsNew;
            obj.Data = interp1(t, obj.Data, tNew, 'linear', 'extrap');
            obj.Fs = fsNew;
        end

        function out = window(obj, t1, t2)
            i1 = max(1, floor(t1*obj.Fs)+1);
            i2 = min(numel(obj.Data), floor(t2*obj.Fs)+1);
            out = h_signal_class(obj.Data(i1:i2), obj.Fs, 'label', obj.Label);
        end

        function disp(obj)
            fprintf('h_signal_class "%s": %d samples @ %g Hz (%s)\n', ...
                obj.Label, obj.NSamples, obj.Fs, obj.Units);
        end

        function plot(obj, varargin)
            t = (0:obj.NSamples-1).' / obj.Fs;
            ax = gca;
            line(t, obj.Data, 'Parent', ax, varargin{:});
            xlabel(ax, 'Time (s)');
            ylabel(ax, sprintf('Amplitude [%s]', obj.Units));
            title(ax, obj.Label);
            grid(ax, 'on');
        end
    end

    methods (Static)
        function obj = fromTable(T)
            obj = h_signal_class(T.value, T.fs(1));
        end
    end

    methods (Access = private)
        function ok = isCompatible(obj, other)
            ok = obj.Fs == other.Fs;
        end
    end
end

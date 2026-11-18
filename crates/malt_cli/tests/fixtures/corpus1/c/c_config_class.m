classdef c_config_class < handle
    %C_CONFIG_CLASS Holds run-time configuration for the simulation.
    %   Values are validated on assignment and can be saved to disk.

    properties
        Name (1,:) char = 'default'
        Iterations (1,1) double {mustBeInteger, mustBePositive} = 100
        Tolerance (1,1) double = 1e-6
        Verbose logical = false
    end

    properties (Dependent)
        Summary
    end

    properties (Constant)
        Version = '2.1.0'
        DefaultFile = 'config_default.mat'
    end

    properties (Access = private)
        Created_
        History = {}
    end

    methods
        function obj = c_config_class(varargin)
            % Constructor accepts name-value pairs for any public property
            obj.Created_ = datetime('now');
            for k = 1:2:numel(varargin)
                obj.(varargin{k}) = varargin{k+1};
            end
        end

        function delete(obj)
            % Log the teardown when verbose
            if obj.Verbose
                fprintf('Config %s released\n', obj.Name);
            end
        end

        function set.Tolerance(obj, value)
            if value <= 0
                error('c_config_class:badTolerance', 'Tolerance must be positive');
            end
            obj.Tolerance = value;
            obj.History{end+1} = sprintf('Tolerance=%g', value);
        end

        function value = get.Summary(obj)
            value = sprintf('%s v%s (%d iterations)', obj.Name, ...
                c_config_class.Version, obj.Iterations);
        end

        function disp(obj)
            % Overloaded display used by the command window
            fprintf('c_config_class: %s\n', obj.Summary);
            fprintf('  tolerance : %g\n', obj.Tolerance);
            fprintf('  verbose   : %d\n', obj.Verbose);
        end

        function save(obj, fileName)
            if nargin < 2
                fileName = c_config_class.DefaultFile;
            end
            settings = struct('Name', obj.Name, 'Iterations', obj.Iterations, ...
                'Tolerance', obj.Tolerance, 'Verbose', obj.Verbose);
            save(fileName, 'settings');
        end
    end

    methods (Static)
        function obj = fromFile(fileName)
            % Build a configuration object from a previously saved file
            if nargin < 1
                fileName = c_config_class.DefaultFile;
            end
            S = load(fileName, 'settings');
            s = S.settings;
            obj = c_config_class('Name', s.Name, 'Iterations', s.Iterations, ...
                'Tolerance', s.Tolerance, 'Verbose', s.Verbose);
        end
    end

    methods (Access = private)
        function n = countHistory(obj)
            n = numel(obj.History);
        end
    end
end

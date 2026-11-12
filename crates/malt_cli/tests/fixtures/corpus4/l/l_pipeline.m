classdef l_pipeline < handle
    %L_PIPELINE Chainable sequence of processing stages.
    %   Stages are function handles or other l_pipeline objects. Each
    %   add() call returns the pipeline, so stages can be chained:
    %
    %       p = l_pipeline().add("norm", @l_helpers1).add("sq", @(x) x.^2);
    %       y = p.run(data);

    properties (SetAccess = private)
        Stages cell = {}
        Names string = string.empty
        Ran logical = false
    end

    properties
        StopOnError (1,1) logical = true
        Verbosity (1,1) double {mustBeInteger, mustBeNonnegative} = 0
    end

    methods
        function obj = l_pipeline(varargin)
            %L_PIPELINE Construct an empty pipeline.
            p = inputParser();
            p.FunctionName = 'l_pipeline';
            p.addParameter('StopOnError', true, @islogical);
            p.addParameter('Verbosity', 0, @(v) isnumeric(v) && isscalar(v));
            p.KeepUnmatched = false;
            p.parse(varargin{:});
            obj.StopOnError = p.Results.StopOnError;
            obj.Verbosity = p.Results.Verbosity;
        end

        function obj = add(obj, name, stage)
            %ADD Append a named stage and return the pipeline.
            arguments
                obj
                name (1,1) string
                stage
            end
            if ~(isa(stage, 'function_handle') || isa(stage, 'l_pipeline'))
                error('l_pipeline:badStage', 'Stage "%s" is not callable.', name);
            end
            obj.Stages{end+1} = stage;
            obj.Names(end+1) = strtrim(name);
        end

        function out = run(obj, input)
            %RUN Feed input through every stage in order.
            out = input;
            obj.Ran = true;
            for k = 1:numel(obj.Stages)
                stage = obj.Stages{k};
                try
                    if isa(stage, 'l_pipeline')
                        out = stage.run(out);
                    else
                        out = stage(out);
                    end
                catch err
                    if obj.StopOnError
                        error('l_pipeline:stageFailed', ...
                            'Stage %s failed: %s', obj.Names(k), err.message);
                    end
                    warning('l_pipeline:stageSkipped', 'Skipping %s.', obj.Names(k));
                end
                if obj.Verbosity > 0
                    fprintf('%s done\n', obj.Names(k));
                end
            end
        end

        function n = numStages(obj)
            n = numel(obj.Stages);
        end

        function obj = clear(obj)
            obj.Stages = {};
            obj.Names = string.empty;
            obj.Ran = false;
        end
    end

    methods (Static)
        function p = standard()
            %STANDARD Pipeline with normalisation and a square stage.
            p = l_pipeline('Verbosity', 1) ...
                .add("normalise", @l_helpers1) ...
                .add("square", @(x) x .^ 2);
        end
    end
end

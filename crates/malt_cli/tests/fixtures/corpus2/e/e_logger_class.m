classdef e_logger_class < handle
    %E_LOGGER_CLASS Simple application logger, one shared instance.
    %   log = e_logger_class.getInstance();
    %   log.Level = e_logger_class.INFO;
    %   log.info('Started %s', datestr(now));
    %
    %   Messages below the current Level are dropped. Output goes to the
    %   console and, when a file is open, to the log file too.

    properties (Constant)
        DEBUG = 10
        INFO = 20
        WARN = 30
        ERROR = 40
    end

    properties
        Level = e_logger_class.INFO
        FileId = -1
    end

    properties (SetAccess = private)
        Count = struct('debug', 0, 'info', 0, 'warn', 0, 'error', 0)
    end

    methods (Access = private)
        function obj = e_logger_class()
            % Private constructor: use getInstance instead.
        end
    end

    methods (Static)
        function obj = getInstance()
            persistent unique
            if isempty(unique) || ~isvalid(unique)
                unique = e_logger_class();
            end
            obj = unique;
        end
    end

    methods
        function openFile(obj, fileName)
            % Open (or append to) a log file.
            if obj.FileId > 0
                fclose(obj.FileId);
            end
            obj.FileId = fopen(fileName, 'a');
            if obj.FileId < 0
                error('e_logger_class:openFailed', 'Cannot open %s', fileName);
            end
            cleanup = onCleanup(@() fprintf('Log file %s ready\n', fileName));
            clear cleanup
        end

        function closeFile(obj)
            if obj.FileId > 0
                fclose(obj.FileId);
                obj.FileId = -1;
            end
        end

        function debug(obj, fmt, varargin)
            obj.write(e_logger_class.DEBUG, 'DEBUG', fmt, varargin{:});
        end

        function info(obj, fmt, varargin)
            obj.write(e_logger_class.INFO, 'INFO', fmt, varargin{:});
        end

        function warn(obj, fmt, varargin)
            obj.write(e_logger_class.WARN, 'WARN', fmt, varargin{:});
        end

        function error(obj, fmt, varargin)
            obj.write(e_logger_class.ERROR, 'ERROR', fmt, varargin{:});
        end

        function disp(obj)
            % Show the current state rather than the default display.
            disp(['e_logger_class: level=' num2str(obj.Level) ...
                  ', file=' num2str(obj.FileId)]);
        end

        function delete(obj)
            obj.closeFile();
        end
    end

    methods (Access = private)
        function write(obj, severity, tag, fmt, varargin)
            if severity < obj.Level
                return
            end
            msg = sprintf(fmt, varargin{:});
            line = sprintf('[%s] %-5s %s\n', datestr(now, 'HH:MM:SS'), tag, msg);
            fprintf(1, '%s', line);
            if obj.FileId > 0
                fprintf(obj.FileId, '%s', line);
            end
            key = lower(tag);
            obj.Count.(key) = obj.Count.(key) + 1;
        end
    end
end

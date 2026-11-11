function varargout = b_app_callbacks(varargin)
% B_APP_CALLBACKS MATLAB code for b_app_callbacks.fig
%      B_APP_CALLBACKS, by itself, creates a new B_APP_CALLBACKS or raises
%      the existing singleton.
%
%      H = B_APP_CALLBACKS returns the handle to the figure.
%
%      B_APP_CALLBACKS('CALLBACK',hObject,eventData,handles,...) calls the
%      local function named CALLBACK in B_APP_CALLBACKS.M.

% Begin initialization code - DO NOT EDIT
gui_Singleton = 1;
gui_State = struct('gui_Name',       mfilename, ...
                   'gui_Singleton',  gui_Singleton, ...
                   'gui_OpeningFcn', @b_app_callbacks_OpeningFcn, ...
                   'gui_OutputFcn',  @b_app_callbacks_OutputFcn, ...
                   'gui_LayoutFcn',  [], ...
                   'gui_Callback',   []);
if nargin && ischar(varargin{1})
    gui_State.gui_Callback = str2func(varargin{1});
end

if nargout
    [varargout{1:nargout}] = gui_mainfcn(gui_State, varargin{:});
else
    gui_mainfcn(gui_State, varargin{:});
end
% End initialization code - DO NOT EDIT


% --- Executes just before b_app_callbacks is made visible.
function b_app_callbacks_OpeningFcn(hObject, eventdata, handles, varargin)
% Default values before the figure is shown
handles.output    = hObject;
handles.threshold = 0.5;
handles.data      = [];
set(handles.thresholdEdit, 'String', num2str(handles.threshold));
guidata(hObject, handles);

% Block until the user closes the window or presses Apply
uiwait(handles.figure1);


% --- Outputs from this function are returned to the command line.
function varargout = b_app_callbacks_OutputFcn(hObject, eventdata, handles)
varargout{1} = handles.output;


% --- Executes on button press in pushbutton1.
function pushbutton1_Callback(hObject, eventdata, handles)
[fileName, pathName] = uigetfile( ...
    {'*.csv', 'CSV files'; '*.mat', 'MAT files'}, 'Open data');
if isequal(fileName, 0)
    return
end

handles.data = readmatrix(fullfile(pathName, fileName));
handles.fileName = fileName;
guidata(hObject, handles);
refreshPlot(handles);


% --- Executes on editing the threshold edit box.
function thresholdEdit_Callback(hObject, eventdata, handles)
val = str2double(get(hObject, 'String'));
if isnan(val) || val < 0 || val > 1
    set(hObject, 'String', num2str(handles.threshold));
    errordlg('Enter a number between 0 and 1.', 'Bad input');
else
    handles.threshold = val;
    guidata(hObject, handles);
    refreshPlot(handles);
end


% --- Executes when user attempts to close figure1.
function figure1_CloseRequestFcn(hObject, eventdata, handles)
if isfield(handles, 'figure1') && isvalid(hObject)
    uiresume(hObject);
end
delete(hObject);


% --- Plot the data and count the points above the threshold.
function refreshPlot(handles)
if isempty(handles.data)
    return
end

x = handles.data(:, 1);
y = handles.data(:, 2);
mask = y > handles.threshold;

ax = handles.axes1;
cla(ax);
plot(ax, x, y, 'b.-');
hold(ax, 'on');
plot(ax, x(mask), y(mask), 'ro', 'MarkerSize', 6);
yline(ax, handles.threshold, 'k--');
hold(ax, 'off');
grid(ax, 'on');

set(handles.countText, 'String', ...
    sprintf('%d of %d points above %.2f', nnz(mask), numel(y), handles.threshold));

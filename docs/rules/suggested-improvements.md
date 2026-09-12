---
icon: lucide/lightbulb
---

# Suggested Improvements

**Default severity:** Info
**Auto-fix:** No
**Category:** Suggested Improvements
**Can be disabled:** Yes

## What this rule does

A data-driven rule module that handles 243 MATLAB Code Analyzer checks
about functions and patterns that are not recommended, pairing each with a
modern replacement suggestion. Instead of implementing one struct per
check, a single `SuggestedImprovementsEngine` loads
`data/suggested_improvements.toml` at compile time, parses it into a
function-name lookup table, and emits diagnostics with the specific check
ID from the matched entry (e.g. `CSVRD`, not the meta ID
`SUGGESTED_IMPROVEMENTS`).

On each `function_call` or `command` node the engine extracts the function
name and performs an O(1) HashMap lookup, reporting the first matching
entry for that name (some names map to several check IDs, e.g.
`maketform` → MTFA1, MTFA2, MTFP1, MTFP2, MTFB). Entries with an empty
`function_name` are "generic" checks that cannot be matched by name alone
(they require AST pattern matching) and are skipped during the
lookup-based check.

## Check IDs

### Suggested Improvements (243 checks)

| Check ID | Severity | Description |
| -------- | -------- | ----------- |
| <a id="importdyn"></a>`IMPORTDYN` | warning | Using function syntax to call 'import' is not recommended. With appropriate code changes, use command syntax instead. |
| <a id="lerr"></a>`LERR` | warning | LASTERR and LASTERROR are not recommended. Use an identifier on the CATCH block instead. |
| <a id="evlc"></a>`EVLC` | warning | Using 'evalc' with two arguments is not recommended. Use try/catch statements instead to make code more clear and efficient. |
| <a id="rand"></a>`RAND` | warning | RAND or RANDN with the 'seed', 'state', or 'twister' inputs is not recommended. Use RNG instead. |
| <a id="hough"></a>`HOUGH` | warning | HOUGH(BW,'ThetaResolution',VAL) is not recommended. Use HOUGH(BW,'Theta',-90:VAL:(90-VAL) ) instead. |
| <a id="thour"></a>`THOUR` | warning | 'hour' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tmnth"></a>`TMNTH` | warning | 'month' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tmnut"></a>`TMNUT` | warning | 'minute' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tnday"></a>`TNDAY` | warning | 'day' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tscnd"></a>`TSCND` | warning | 'second' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tqurt"></a>`TQURT` | warning | 'quarter' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tyear"></a>`TYEAR` | warning | 'year' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tdtvec"></a>`TDTVEC` | warning | 'datevec' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' instead. |
| <a id="tnow1"></a>`TNOW1` | warning | 'now' is not recommended. With appropriate code changes, use 'datetime(\ |
| <a id="tnow2"></a>`TNOW2` | warning | 'datetime(now, 'ConvertFrom', 'datenum')' is not recommended. Use 'datetime(\ |
| <a id="ttday1"></a>`TTDAY1` | warning | 'today' is not recommended. With appropriate code changes, use 'datetime(\ |
| <a id="ttday2"></a>`TTDAY2` | warning | 'datetime(today, 'ConvertFrom', 'datenum')' is not recommended. Use 'datetime(\ |
| <a id="match2"></a>`MATCH2` | warning | STRMATCH is not recommended. Use STRNCMP or VALIDATESTRING instead. |
| <a id="imgdt"></a>`IMGDT` | warning | Using 'DataAugmentation' in function 'imageInputLayer' is not recommended. Use function 'augmentedImageDatastore' instead. |
| <a id="match3"></a>`MATCH3` | warning | STRMATCH is not recommended. Use STRCMP instead. |
| <a id="mtfa1"></a>`MTFA1` | warning | MAKETFORM('AFFINE',A) is not recommended. Use AFFINE2D or AFFINE3D instead. |
| <a id="mtfa2"></a>`MTFA2` | warning | MAKETFORM('AFFINE',U,X) is not recommended. Use FITGEOTRANS instead. |
| <a id="mtfp1"></a>`MTFP1` | warning | MAKETFORM('PROJECTIVE',A) is not recommended. Use PROJECTIVE2D instead. |
| <a id="mtfp2"></a>`MTFP2` | warning | MAKETFORM('PROJECTIVE',U,X) is not recommended. Use FITGEOTRANS instead. |
| <a id="mtfb"></a>`MTFB` | warning | MAKETFORM('BOX',...) is not recommended. Use IMREF2D or IMREF3D instead. |
| <a id="oops"></a>`OOPS` | warning | Defining a class using 'function' syntax is not recommended. With appropriate code changes, use 'classdef' syntax instead. |
| <a id="pmtmconf"></a>`PMTMCONF` | warning | When using PMTM with three output arguments, the 'ConfidenceLevel' input argument is recommended. |
| <a id="nchki"></a>`NCHKI` | warning | NARGCHK is not recommended. Use NARGINCHK instead. |
| <a id="nchko"></a>`NCHKO` | warning | Using NARGCHK with NARGOUT is not recommended. Use NARGOUTCHK instead. |
| <a id="nchkn"></a>`NCHKN` | warning | NARGCHK is not recommended. Use NARGINCHK without ERROR instead. |
| <a id="nchkm"></a>`NCHKM` | warning | NARGCHK is not recommended. Use NARGOUTCHK without ERROR instead. |
| <a id="isclstr"></a>`ISCLSTR` | warning | To support string in addition to cellstr, include a call to 'isstring'. |
| <a id="emtag"></a>`EMTAG` | warning | The compilation directive (or pragma) 'eml' is not recommended. Use 'codegen' instead. |
| <a id="emxtr"></a>`EMXTR` | warning | The 'eml' namespace is not recommended. Use 'codegen' instead. |
| <a id="nvrepla"></a>`NVREPLA` | warning | 'addParamValue' is not recommended. Use 'addParameter' instead. |
| <a id="nvreplm"></a>`NVREPLM` | warning | 'MidPctRef' is not recommended. Use 'MidPercentReferenceLevel' instead. |
| <a id="nvreplp"></a>`NVREPLP` | warning | 'PctRefLevels' is not recommended. Use 'PercentReferenceLevels' instead. |
| <a id="vidread"></a>`VIDREAD` | warning | 'NumberOfFrames' is not recommended. Use 'NumFrames' instead. |
| <a id="crnr"></a>`CRNR` | warning | CORNER is not recommended. Use detectHarrisFeatures or detectMinEigenFeatures in Computer Vision Toolbox instead. |
| <a id="crnrm"></a>`CRNRM` | warning | CORNERMETRIC is not recommended. Use detectHarrisFeatures or detectMinEigenFeatures and the cornerPoints class in Computer Vision Toolbox instead. |
| <a id="mdflt1"></a>`MDFLT1` | warning | BLKSZ is required for backward compatibility and is ignored. Use [] instead. |
| <a id="intrpp"></a>`INTRPP` | warning | 'pp' is not recommended. Use the griddedInterpolant class instead. |
| <a id="displayprog"></a>`DISPLAYPROG` | warning | Programmatic use of DISPLAY is not recommended. Use DISP or FPRINTF instead. |
| <a id="hgstgt"></a>`HGSTGT` | warning | hgsetget is not recommended. Use matlab.mixin.SetGet or matlab.mixin.SetGetExactNames instead. |
| <a id="legacymd"></a>`LEGACYMD` | warning | Setting LegacyMode to true is not recommended. Set LegacyMode to false instead. |
| <a id="legacytrd"></a>`LEGACYTRD` | warning | 'DetectorMethod' is applicable only when LegacyMode is set to true. However, setting LegacyMode to true is not recommended. |
| <a id="legacytrl"></a>`LEGACYTRL` | warning | 'LoopMethod' is applicable only when LegacyMode is set to true. However, setting LegacyMode to true is not recommended. |
| <a id="legacytru"></a>`LEGACYTRU` | warning | 'UpdatePeriod' is applicable only when LegacyMode is set to true. However, setting LegacyMode to true is not recommended. |
| <a id="legacytrs"></a>`LEGACYTRS` | warning | 'StepSize' is applicable only when LegacyMode is set to true. However, setting LegacyMode to true is not recommended. |
| <a id="legacytrg"></a>`LEGACYTRG` | warning | 'GainOutputPort' is applicable only when LegacyMode is set to true. However, setting LegacyMode to true is not recommended. |
| <a id="ezplt"></a>`EZPLT` | warning | EZPLOT is not recommended. Use FPLOT or FIMPLICIT instead. |
| <a id="ezgrph3"></a>`EZGRPH3` | warning | EZGRAPH3 is not recommended. Use FCONTOUR, FMESH, FPLOT, FPLOT3 or FSURF instead. |
| <a id="ezcntrf"></a>`EZCNTRF` | warning | EZCONTOURF is not recommended. Use FCONTOUR instead, and set the 'Fill' value to 'on'. |
| <a id="ezmshc"></a>`EZMSHC` | warning | EZMESHC is not recommended. Use FMESH instead, and set the 'ShowContours' value to 'on'. |
| <a id="ezsrfc"></a>`EZSRFC` | warning | EZSURFC is not recommended. Use FSURF instead, and set the 'ShowContours' value to 'on'. |
| <a id="fisadr"></a>`FISADR` | warning | 'addrule' is not recommended. Use 'addRule' instead. |
| <a id="strquot"></a>`STRQUOT` | warning | string('...') is not recommended. Use \ |
| <a id="strclqt"></a>`STRCLQT` | warning | 'string({'str1', 'str2'})' is not recommended. Use '[\ |
| <a id="sim"></a>`SIM` | warning | 'sim' in parfor loop is not recommended. Replace the parfor loop with 'parsim'. |
| <a id="numch"></a>`NUMCH` | warning | 'NumberOfChannels' is not recommended. Use 'NumChannels' instead. |
| <a id="gtred"></a>`GTRED` | warning | 'geotiffread' is not recommended, except when reading a GeoTIFF file from a URL. With appropriate code changes, use 'readgeoraster' instead. |
| <a id="getfsp"></a>`GETFSP` | warning | Using get for retrieving values of line spacing is not recommended. With appropriate code changes, use 'settings' object instead. |
| <a id="setfmt"></a>`SETFMT` | warning | Using set for assigning values of numeric display format is not recommended. With appropriate code changes, use 'settings' object instead. |
| <a id="setfsp"></a>`SETFSP` | warning | Using set for assigning values of line spacing is not recommended. With appropriate code changes, use 'settings' object instead. |
| <a id="getfmt"></a>`GETFMT` | warning | Using get for retrieving values of numeric display format is not recommended. With appropriate code changes, use 'settings' object instead. |
| <a id="ev2in"></a>`EV2IN` | warning | Using 'eval' with two arguments is not recommended. Use try/catch statements instead to make code more clear and efficient. |
| <a id="ev3in"></a>`EV3IN` | warning | Using 'evalin' with three arguments is not recommended. Use try/catch statements instead to make code more clear and efficient. |
| <a id="prtog"></a>`PRTOG` | warning | '-opengl' is not recommended. Use '-image' instead, which is a direct replacement. |
| <a id="prtpt"></a>`PRTPT` | warning | '-painters' is not recommended. Use '-vector' instead, which is a direct replacement. |
| <a id="formatnoi"></a>`FORMATNOI` | warning | 'format' with no input or output arguments is not recommended. Use 'format(\ |
| <a id="xfrwsb"></a>`XFRWSB` | warning | The 'TransferBaseWorkspaceVariables' option is not recommended for 'batchsim'. With appropriate code changes, consider using project startup scripts or the 'SetupFcn' option instead. |
| <a id="xfrwsp"></a>`XFRWSP` | warning | The 'TransferBaseWorkspaceVariables' option is not recommended for 'parsim'. With appropriate code changes, consider using project startup scripts or the 'SetupFcn' option instead. |
| <a id="oldsim"></a>`OLDSIM` | warning | This syntax of the 'sim' command which returns multiple arguments is not recommended. With appropriate code changes, turn on 'ReturnWorkspaceOutputs' and return simulation results using the single-output format instead. |
| <a id="insthwi"></a>`INSTHWI` | warning | 'instrhwinfo('ivi')' is not recommended. With appropriate code changes, use 'ividriverlist' or 'ividevlist' instead. |
| <a id="inwvx"></a>`INWVX` | warning | 'instrhwinfo('vxipnp')' is not recommended. With appropriate code changes, use 'ividriverlist' or 'ividevlist' instead. |
| <a id="vermatlab"></a>`VERMATLAB` | warning | ver('matlab') is not recommended. With appropriate code changes, use 'matlabRelease' instead. |
| <a id="verlessmatlab"></a>`VERLESSMATLAB` | warning | verLessThan('matlab', ...) is not recommended. With appropriate code changes, use 'isMATLABReleaseOlderThan' instead. |
| <a id="risklmm"></a>`RISKLMM` | warning | 'Model' property of 'risk.credit.pd.LifetimePDModel' class is not recommended. Use 'UnderlyingModel' property of 'risk.credit.pd.LifetimePDModel' class instead. |
| <a id="risklma"></a>`RISKLMA` | warning | 'modelAccuracy' method of 'risk.credit.pd.LifetimePDModel' class is not recommended. Use 'modelCalibration' method of 'risk.credit.pd.LifetimePDModel' class instead. |
| <a id="risklmap"></a>`RISKLMAP` | warning | 'modelAccuracyPlot' method of 'risk.credit.pd.LifetimePDModel' class is not recommended. Use 'modelCalibrationPlot' method of 'risk.credit.pd.LifetimePDModel' class instead. |
| <a id="risklgdma"></a>`RISKLGDMA` | warning | 'modelAccuracy' method of 'risk.credit.lgd.LGDModel' class is not recommended. Use 'modelCalibration' method of 'risk.credit.lgd.LGDModel' class instead. |
| <a id="risklgdmap"></a>`RISKLGDMAP` | warning | 'modelAccuracyPlot' method of 'risk.credit.lgd.LGDModel' class is not recommended. Use 'modelCalibrationPlot' method of 'risk.credit.lgd.LGDModel' class instead. |
| <a id="riskeadma"></a>`RISKEADMA` | warning | 'modelAccuracy' method of 'risk.credit.ead.EADModel' class is not recommended. Use 'modelCalibration' method of 'risk.credit.ead.EADModel' class instead. |
| <a id="riskeadmap"></a>`RISKEADMAP` | warning | 'modelAccuracyPlot' method of 'risk.credit.ead.EADModel' class is not recommended. Use 'modelCalibrationPlot' method of 'risk.credit.ead.EADModel' class instead. |
| <a id="holdall"></a>`HOLDALL` | warning | 'hold('all')' is not recommended. Use 'hold('on')' instead, which is a direct replacement. |
| <a id="masso"></a>`MASSO` | warning | 'MasterSolverOptions' is not recommended. Use 'MainSolverOptions' instead, which is a direct replacement. |
| <a id="imsso"></a>`IMSSO` | warning | 'IntMasterSolverOptions' is not recommended. Use 'IntMainSolverOptions' instead, which is a direct replacement. |
| <a id="cdfepoch2date"></a>`CDFEPOCH2DATE` | warning | 'ConvertEpochToDatenum' is not recommended. With appropriate code changes, use the 'DatetimeType' parameter of 'cdfread' instead. |
| <a id="featgpid"></a>`FEATGPID` | warning | 'feature('getpid')' is unsupported and not recommended. With appropriate code changes, use the function 'matlabProcessID' instead. |
| <a id="dtrirep"></a>`DTRIREP` | warning | 'TriRep' is not recommended. With appropriate code changes, use 'triangulation' instead. |
| <a id="ddeltri"></a>`DDELTRI` | warning | 'DelaunayTri' is not recommended. With appropriate code changes, use 'delaunayTriangulation' instead. |
| <a id="dtriint"></a>`DTRIINT` | warning | 'TriScatteredInterp' is not recommended. With appropriate code changes, use 'scatteredInterpolant' instead. |
| <a id="dapplut"></a>`DAPPLUT` | warning | 'applylut' is not recommended. With appropriate code changes, use 'bwlookup' instead. |
| <a id="dblkprc"></a>`DBLKPRC` | warning | 'blkproc' is not recommended. With appropriate code changes, use 'blockproc' instead. |
| <a id="caxis"></a>`CAXIS` | warning | 'caxis' is not recommended. Use 'clim' instead, which is a direct replacement. |
| <a id="cdfepoch"></a>`CDFEPOCH` | warning | 'cdfepoch' is not recommended. With appropriate code changes, use 'cdflib' low-level functions instead. |
| <a id="todatenum"></a>`TODATENUM` | warning | 'todatenum' is not recommended. With appropriate code changes, use the 'DatetimeType' parameter of 'cdfread' instead. |
| <a id="commpamm"></a>`COMMPAMM` | warning | 'comm.PAMModulator' is not recommended. With appropriate code changes, use 'pammod' instead. |
| <a id="commpamd"></a>`COMMPAMD` | warning | 'comm.PAMDemodulator' is not recommended. With appropriate code changes, use 'pamdemod' instead. |
| <a id="commsrc"></a>`COMMSRC` | warning | 'commsrc.pn' is not recommended. With appropriate code changes, use 'comm.PNSequence' instead. |
| <a id="dcptf"></a>`DCPTF` | warning | 'cp2tform' is not recommended. With appropriate code changes, use 'fitgeotrans' instead. |
| <a id="datnm"></a>`DATNM` | warning | 'datenum' is not recommended. With appropriate code changes, use 'datetime' instead. |
| <a id="datst"></a>`DATST` | warning | 'datestr' is not recommended. With appropriate code changes, use 'datetime' instead. |
| <a id="detim"></a>`DETIM` | warning | 'etime' is not recommended. With appropriate code changes, use 'datetime' and the minus operator instead. |
| <a id="datod"></a>`DATOD` | warning | 'addtodate' is not recommended. With appropriate code changes, use 'datetime', 'duration', and the plus operator instead. |
| <a id="clock"></a>`CLOCK` | warning | 'clock' is not recommended. With appropriate code changes, use 'datetime(\ |
| <a id="date"></a>`DATE` | warning | 'date' is not recommended. With appropriate code changes, use 'datetime(\ |
| <a id="datic"></a>`DATIC` | warning | 'datetick' is not recommended. With appropriate code changes, use datetime and duration arrays directly in charts. Modify display using 'xtickformat', 'ytickformat', or 'ztickformat'. |
| <a id="wknum"></a>`WKNUM` | warning | 'weeknum' is not recommended. With appropriate code changes, use 'week' with a 'datetime' input instead. |
| <a id="emdate"></a>`EMDATE` | warning | 'eomdate' is not recommended. With appropriate code changes, use 'dateshift' with a 'datetime' input instead. |
| <a id="xmdate"></a>`XMDATE` | warning | 'x2mdate' is not recommended. With appropriate code changes, use 'datetime(..., \ |
| <a id="mxdate"></a>`MXDATE` | warning | 'm2xdate' is not recommended. With appropriate code changes, use 'exceltime' with a 'datetime' input instead. |
| <a id="mnths"></a>`MNTHS` | warning | 'months' is not recommended. With appropriate code changes, use 'between' with a 'datetime' input instead. |
| <a id="dgtord"></a>`DGTORD` | warning | 'degtorad' is not recommended. Use 'deg2rad' instead, which is a direct replacement. |
| <a id="rdtodg"></a>`RDTODG` | warning | 'radtodeg' is not recommended. Use 'rad2deg' instead, which is a direct replacement. |
| <a id="ezcontr"></a>`EZCONTR` | warning | 'ezcontour' is not recommended. With appropriate code changes, use 'fcontour' instead. |
| <a id="ezmesh"></a>`EZMESH` | warning | 'ezmesh' is not recommended. With appropriate code changes, use 'fmesh' instead. |
| <a id="ezplt3"></a>`EZPLT3` | warning | 'ezplot3' is not recommended. With appropriate code changes, use 'fplot3' instead. |
| <a id="ezsurf"></a>`EZSURF` | warning | 'ezsurf' is not recommended. With appropriate code changes, use 'fsurf' instead. |
| <a id="ezpolar"></a>`EZPOLAR` | warning | 'ezpolar' is not recommended. With appropriate code changes, use 'fpolarplot' instead. |
| <a id="compass"></a>`COMPASS` | warning | 'compass' is not recommended. With appropriate code changes, use 'compassplot' instead. |
| <a id="dflipdim"></a>`DFLIPDIM` | warning | 'flipdim' is not recommended. With appropriate code changes, use 'flip' instead. |
| <a id="dstrmt"></a>`DSTRMT` | warning | 'str2mat' is not recommended. With appropriate code changes, use 'char' instead. |
| <a id="dststr"></a>`DSTSTR` | warning | 'setstr' is not recommended. With appropriate code changes, use 'char' instead. |
| <a id="dstrvct"></a>`DSTRVCT` | warning | 'strvcat' is not recommended. With appropriate code changes, use 'char' instead. |
| <a id="disstr"></a>`DISSTR` | warning | 'isstr' is not recommended. With appropriate code changes, use 'ischar' instead. |
| <a id="dftsmtx"></a>`DFTSMTX` | warning | 'fts2mtx' is not recommended. With appropriate code changes, use 'fts2mat' instead. |
| <a id="fiswrt"></a>`FISWRT` | warning | 'writefis' is not recommended. Use 'writeFIS' instead, which is a direct replacement. |
| <a id="fisadm"></a>`FISADM` | warning | 'addmf' is not recommended. With appropriate code changes, use 'addMF' instead. |
| <a id="hist"></a>`HIST` | warning | 'hist' is not recommended. With appropriate code changes, use 'histogram' instead. |
| <a id="histc"></a>`HISTC` | warning | 'histc' is not recommended. With appropriate code changes, use 'histcounts' instead. |
| <a id="rose"></a>`ROSE` | warning | 'rose' is not recommended. With appropriate code changes, use 'polarhistogram' instead. |
| <a id="h5cls"></a>`H5CLS` | warning | 'H5.close' is not recommended and no longer has any effect. There is no simple replacement for this. |
| <a id="h5opn"></a>`H5OPN` | warning | 'H5.open' is not recommended and no longer has any effect. There is no simple replacement for this. |
| <a id="hdfi"></a>`HDFI` | warning | 'hdf5info' is not recommended. With appropriate code changes, use 'h5info' instead. |
| <a id="hdfw"></a>`HDFW` | warning | 'hdf5write' is not recommended. With appropriate code changes, use 'h5write' instead. |
| <a id="hdfr"></a>`HDFR` | warning | 'hdf5read' is not recommended. With appropriate code changes, use 'h5read' instead. |
| <a id="im2bw"></a>`IM2BW` | warning | 'im2bw' is not recommended. With appropriate code changes, use 'imbinarize' instead. |
| <a id="ispix"></a>`ISPIX` | warning | 'pixelLabelImageSource' is not recommended. Use 'pixelLabelImageDatastore' instead, which is a direct replacement. |
| <a id="isaug"></a>`ISAUG` | warning | 'augmentedImageSource' is not recommended. Use 'augmentedImageDatastore' instead, which is a direct replacement. |
| <a id="isdns"></a>`ISDNS` | warning | 'denoisingImageSource' is not recommended. Use 'denoisingImageDatastore' instead, which is a direct replacement. |
| <a id="imfreeh"></a>`IMFREEH` | warning | 'imfreehand' is not recommended. With appropriate code changes, use 'drawfreehand' instead. |
| <a id="imrect"></a>`IMRECT` | warning | 'imrect' is not recommended. With appropriate code changes, use 'drawrectangle' instead. |
| <a id="imline"></a>`IMLINE` | warning | 'imline' is not recommended. With appropriate code changes, use 'drawline' instead. |
| <a id="impnt"></a>`IMPNT` | warning | 'impoint' is not recommended. With appropriate code changes, use 'drawpoint' instead. |
| <a id="impoly"></a>`IMPOLY` | warning | 'impoly' is not recommended. With appropriate code changes, use 'drawpolygon' or 'drawpolyline' instead. |
| <a id="imellps"></a>`IMELLPS` | warning | 'imellipse' is not recommended. With appropriate code changes, use 'drawellipse' or 'drawcircle' instead. |
| <a id="dimtrns"></a>`DIMTRNS` | warning | 'imtransform' is not recommended. With appropriate code changes, use 'imwarp' instead. |
| <a id="fileattrib"></a>`FILEATTRIB` | warning | 'fileattrib' is not recommended. With appropriate code changes, use 'filePermissions' instead. |
| <a id="csvrd"></a>`CSVRD` | warning | 'csvread' is not recommended. With appropriate code changes, use 'readtable' or 'readmatrix' instead. |
| <a id="dlmrd"></a>`DLMRD` | warning | 'dlmread' is not recommended. With appropriate code changes, use 'readtable' or 'readmatrix' instead. |
| <a id="csvwt"></a>`CSVWT` | warning | 'csvwrite' is not recommended. With appropriate code changes, use 'writematrix' instead. |
| <a id="dlmwt"></a>`DLMWT` | warning | 'dlmwrite' is not recommended. With appropriate code changes, use 'writematrix' instead. |
| <a id="xlsrd"></a>`XLSRD` | warning | 'xlsread' is not recommended. With appropriate code changes, use 'readtable', 'readmatrix' or 'readcell' instead. |
| <a id="xlswt"></a>`XLSWT` | warning | 'xlswrite' is not recommended. With appropriate code changes, use 'writematrix' or 'writecell' instead. |
| <a id="isdir"></a>`ISDIR` | warning | 'isdir' is not recommended. Use 'isfolder' instead, which is a direct replacement. |
| <a id="diseqn"></a>`DISEQN` | warning | 'isequalwithequalnans' is not recommended. With appropriate code changes, use 'isequaln' instead. |
| <a id="dgcat"></a>`DGCAT` | warning | 'gcat' is not recommended. Use 'spmdCat' instead, which is a direct replacement. |
| <a id="dgop"></a>`DGOP` | warning | 'gop' is not recommended. Use 'spmdReduce' instead, which is a direct replacement. |
| <a id="dgplus"></a>`DGPLUS` | warning | 'gplus' is not recommended. Use 'spmdPlus' instead, which is a direct replacement. |
| <a id="dlabbarrier"></a>`DLABBARRIER` | warning | 'labBarrier' is not recommended. Use 'spmdBarrier' instead, which is a direct replacement. |
| <a id="dlabbroadcast"></a>`DLABBROADCAST` | warning | 'labBroadcast' is not recommended. Use 'spmdBroadcast' instead, which is a direct replacement. |
| <a id="dlabindex"></a>`DLABINDEX` | warning | 'labindex' is not recommended. Use 'spmdIndex' instead, which is a direct replacement. |
| <a id="dlabprobe"></a>`DLABPROBE` | warning | 'labProbe' is not recommended. Use 'spmdProbe' instead, which is a direct replacement. |
| <a id="dlabreceive"></a>`DLABRECEIVE` | warning | 'labReceive' is not recommended. Use 'spmdReceive' instead, which is a direct replacement. |
| <a id="dlabsend"></a>`DLABSEND` | warning | 'labSend' is not recommended. Use 'spmdSend' instead, which is a direct replacement. |
| <a id="dlabsendreceive"></a>`DLABSENDRECEIVE` | warning | 'labSendReceive' is not recommended. Use 'spmdSendReceive' instead, which is a direct replacement. |
| <a id="dnumlabs"></a>`DNUMLABS` | warning | 'numlabs' is not recommended. Use 'spmdSize' instead, which is a direct replacement. |
| <a id="agred"></a>`AGRED` | warning | 'arcgridread' is not recommended. With appropriate code changes, use 'readgeoraster' instead. |
| <a id="mdfobj"></a>`MDFOBJ` | warning | 'mdf' is not recommended. With appropriate code changes, use 'mdfInfo', 'mdfChannelGroupInfo' or 'mdfChannelInfo' instead. |
| <a id="ddbmex"></a>`DDBMEX` | warning | 'mexdebug' is not recommended. With appropriate code changes, use 'dbmex' instead. |
| <a id="mlnt"></a>`MLNT` | warning | 'mlint' is not recommended. Use 'checkcode' instead, which is a direct replacement. |
| <a id="mnrfit"></a>`MNRFIT` | warning | 'mnrfit' is not recommended. With appropriate code changes, use 'fitmnr' instead. |
| <a id="mnrval"></a>`MNRVAL` | warning | 'mnrval' is not recommended. With appropriate code changes, use 'MultinomialRegression.predict' instead. |
| <a id="mustinrange"></a>`MUSTINRANGE` | warning | 'mustBeInRange' is not recommended. With appropriate code changes, use 'mustBeBetween' instead. |
| <a id="depnoe"></a>`DEPNOE` | warning | 'numberofelements' is not recommended. With appropriate code changes, use 'numel' instead. |
| <a id="gaopt"></a>`GAOPT` | warning | 'gaoptimset' is not recommended. With appropriate code changes, use 'optimoptions' instead. |
| <a id="psopt"></a>`PSOPT` | warning | 'psoptimset' is not recommended. With appropriate code changes, use 'optimoptions' instead. |
| <a id="saopt"></a>`SAOPT` | warning | 'saoptimset' is not recommended. With appropriate code changes, use 'optimoptions' instead. |
| <a id="plotyy"></a>`PLOTYY` | warning | 'plotyy' is not recommended. With appropriate code changes, use 'yyaxis' instead. |
| <a id="polar"></a>`POLAR` | warning | 'polar' (MATLAB) is not recommended. Use 'polarplot' instead. |
| <a id="plbl"></a>`PLBL` | warning | 'polybool' is not recommended. With appropriate code changes, use 'polyshape' instead. |
| <a id="pyver"></a>`PYVER` | warning | 'pyversion' is not recommended. With appropriate code changes, use 'pyenv' instead. |
| <a id="dquad"></a>`DQUAD` | warning | 'quad' is not recommended. With appropriate code changes, use 'integral' instead. |
| <a id="dquadl"></a>`DQUADL` | warning | 'quadl' is not recommended. With appropriate code changes, use 'integral' instead. |
| <a id="dquadv"></a>`DQUADV` | warning | 'quadv' is not recommended. With appropriate code changes, use 'integral' instead. |
| <a id="ddblqd"></a>`DDBLQD` | warning | 'dblquad' is not recommended. With appropriate code changes, use 'integral2' instead. |
| <a id="dtriqd"></a>`DTRIQD` | warning | 'triplequad' is not recommended. With appropriate code changes, use 'integral3' instead. |
| <a id="dfirrcos"></a>`DFIRRCOS` | warning | 'firrcos' is not recommended. With appropriate code changes, use 'rcosdesign' instead. |
| <a id="dfirgauss"></a>`DFIRGAUSS` | warning | 'firgauss' is not recommended. With appropriate code changes, use 'gaussdesign' instead. |
| <a id="dgaussfir"></a>`DGAUSSFIR` | warning | 'gaussfir' is not recommended. With appropriate code changes, use 'gaussdesign' instead. |
| <a id="roifill"></a>`ROIFILL` | warning | 'roifill' is not recommended. With appropriate code changes, use 'regionfill' instead. |
| <a id="depbart"></a>`DEPBART` | warning | 'sigwin.barthannwin' is not recommended. With appropriate code changes, use 'barthannwin' instead. |
| <a id="deplett"></a>`DEPLETT` | warning | 'sigwin.bartlett' is not recommended. With appropriate code changes, use 'bartlett' instead. |
| <a id="dblkmn"></a>`DBLKMN` | warning | 'sigwin.blackman' is not recommended. With appropriate code changes, use 'blackman' instead. |
| <a id="dbhrrs"></a>`DBHRRS` | warning | 'sigwin.blackmanharris' is not recommended. With appropriate code changes, use 'blackmanharris' instead. |
| <a id="dbhmnwn"></a>`DBHMNWN` | warning | 'sigwin.bohmanwin' is not recommended. With appropriate code changes, use 'bohmanwin' instead. |
| <a id="dchbwn"></a>`DCHBWN` | warning | 'sigwin.chebwin' is not recommended. With appropriate code changes, use 'chebwin' instead. |
| <a id="dflttpwn"></a>`DFLTTPWN` | warning | 'sigwin.flattopwin' is not recommended. With appropriate code changes, use 'flattopwin' instead. |
| <a id="dgswin"></a>`DGSWIN` | warning | 'sigwin.gausswin' is not recommended. With appropriate code changes, use 'gausswin' instead. |
| <a id="dhmmng"></a>`DHMMNG` | warning | 'sigwin.hamming' is not recommended. With appropriate code changes, use 'hamming' instead. |
| <a id="dhann"></a>`DHANN` | warning | 'sigwin.hann' is not recommended. With appropriate code changes, use 'hann' instead. |
| <a id="dkser"></a>`DKSER` | warning | 'sigwin.kaiser' is not recommended. With appropriate code changes, use 'kaiser' instead. |
| <a id="dnlwn"></a>`DNLWN` | warning | 'sigwin.nuttallwin' is not recommended. With appropriate code changes, use 'nuttallwin' instead. |
| <a id="dpnwn"></a>`DPNWN` | warning | 'sigwin.parzenwin' is not recommended. With appropriate code changes, use 'parzenwin' instead. |
| <a id="drctwn"></a>`DRCTWN` | warning | 'sigwin.rectwin' is not recommended. With appropriate code changes, use 'rectwin' instead. |
| <a id="dtylrwn"></a>`DTYLRWN` | warning | 'sigwin.taylorwin' is not recommended. With appropriate code changes, use 'taylorwin' instead. |
| <a id="dtrng"></a>`DTRNG` | warning | 'sigwin.triang' is not recommended. With appropriate code changes, use 'triang' instead. |
| <a id="dtkywn"></a>`DTKYWN` | warning | 'sigwin.tukeywin' is not recommended. With appropriate code changes, use 'tukeywin' instead. |
| <a id="dburg"></a>`DBURG` | warning | 'spectrum.burg' is not recommended. With appropriate code changes, use 'pburg' instead. |
| <a id="dcov"></a>`DCOV` | warning | 'spectrum.cov' is not recommended. With appropriate code changes, use 'pcov' instead. |
| <a id="devctr"></a>`DEVCTR` | warning | 'spectrum.eigenvector' is not recommended. With appropriate code changes, use 'peig' instead. |
| <a id="dmcov"></a>`DMCOV` | warning | 'spectrum.mcov' is not recommended. With appropriate code changes, use 'pmcov' instead. |
| <a id="dmtm"></a>`DMTM` | warning | 'spectrum.mtm' is not recommended. With appropriate code changes, use 'pmtm' instead. |
| <a id="dmusic"></a>`DMUSIC` | warning | 'spectrum.music' is not recommended. With appropriate code changes, use 'pmusic' instead. |
| <a id="dprdgrm"></a>`DPRDGRM` | warning | 'spectrum.periodogram' is not recommended. With appropriate code changes, use 'periodogram' instead. |
| <a id="dwelch"></a>`DWELCH` | warning | 'spectrum.welch' is not recommended. With appropriate code changes, use 'pwelch' instead. |
| <a id="dyulear"></a>`DYULEAR` | warning | 'spectrum.yulear' is not recommended. With appropriate code changes, use 'pyulear' instead. |
| <a id="simvariant"></a>`SIMVARIANT` | warning | 'Simulink.Variant' is not recommended. Use 'Simulink.VariantExpression' instead, which is a direct replacement. |
| <a id="combnk"></a>`COMBNK` | warning | 'combnk' is not recommended. With appropriate code changes, use 'nchoosek' instead. |
| <a id="nanmean"></a>`NANMEAN` | warning | 'nanmean' is not recommended. With appropriate code changes, use 'mean' instead. |
| <a id="nanmedian"></a>`NANMEDIAN` | warning | 'nanmedian' is not recommended. With appropriate code changes, use 'median' instead. |
| <a id="nanmax"></a>`NANMAX` | warning | 'nanmax' is not recommended. With appropriate code changes, use 'max' instead. |
| <a id="nanmin"></a>`NANMIN` | warning | 'nanmin' is not recommended. With appropriate code changes, use 'min' instead. |
| <a id="nanstd"></a>`NANSTD` | warning | 'nanstd' is not recommended. With appropriate code changes, use 'std' instead. |
| <a id="nanvar"></a>`NANVAR` | warning | 'nanvar' is not recommended. With appropriate code changes, use 'var' instead. |
| <a id="nancov"></a>`NANCOV` | warning | 'nancov' is not recommended. With appropriate code changes, use 'cov' instead. |
| <a id="nansum"></a>`NANSUM` | warning | 'nansum' is not recommended. With appropriate code changes, use 'sum' instead. |
| <a id="celldtset"></a>`CELLDTSET` | warning | 'cell2dataset' is not recommended. With appropriate code changes, use 'cell2table' instead. |
| <a id="dtset"></a>`DTSET` | warning | 'dataset' is not recommended. With appropriate code changes, use 'table' instead. |
| <a id="matdtset"></a>`MATDTSET` | warning | 'mat2dataset' is not recommended. With appropriate code changes, use 'array2table' instead. |
| <a id="structdtset"></a>`STRUCTDTSET` | warning | 'struct2dataset' is not recommended. With appropriate code changes, use 'struct2table' instead. |
| <a id="fstr"></a>`FSTR` | warning | 'findstr' is not recommended. With appropriate code changes, use 'strfind' instead. |
| <a id="dstrrd"></a>`DSTRRD` | warning | 'strread' is not recommended. With appropriate code changes, use 'textscan' instead. |
| <a id="dtxtrd"></a>`DTXTRD` | warning | 'textread' is not recommended. With appropriate code changes, use 'textscan' instead. |
| <a id="subimgnr"></a>`SUBIMGNR` | warning | 'subimage' is not recommended. With appropriate code changes, use 'imshow' instead. |
| <a id="urlwr"></a>`URLWR` | warning | 'urlwrite' is not recommended. With appropriate code changes, use 'websave' instead. |
| <a id="urlrd"></a>`URLRD` | warning | 'urlread' is not recommended. With appropriate code changes, use 'webread' or 'webwrite' instead. |
| <a id="vemat"></a>`VEMAT` | warning | 'vec2mat' is not recommended. With appropriate code changes, use 'reshape' instead. |
| <a id="mdfrd"></a>`MDFRD` | warning | 'read' method of 'mdf' class is not recommended. With appropriate code changes, use 'mdfRead' function instead. |
| <a id="mdfchl"></a>`MDFCHL` | warning | 'channelList' method of 'mdf' class is not recommended. With appropriate code changes, use 'mdfChannelInfo' function instead. |
| <a id="mdfsva"></a>`MDFSVA` | warning | 'saveAttachment' method of 'mdf' class is not recommended. With appropriate code changes, use 'mdfSaveAttachment' function instead. |
| <a id="minelv"></a>`MINELV` | warning | 'MinElevationAngle' is not recommended. Use 'MaskElevationAngle' instead, which is a direct replacement when specified as scalar values. Row vector values must be transposed. |

## Examples

### Incorrect

```matlab
data = csvread('data.csv');   % CSVRD — use readmatrix instead
if isdir(folder)              % ISDIR — use isfolder instead
    disp('folder exists');
end
```

### Correct

```matlab
data = readmatrix('data.csv');
if isfolder(folder)
    disp('folder exists');
end
```

## Configuration

```toml
[lint.categories]
suggested-improvements = "info"

[lint.rules]
SUGGESTED_IMPROVEMENTS = "off"
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.

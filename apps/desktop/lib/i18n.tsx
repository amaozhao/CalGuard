'use client';

import { createContext, useContext, useEffect, useMemo, useState } from 'react';

export type Language = 'en' | 'zh';

const storageKey = 'calguard.language';

const dictionaries = {
  en: {
    active: 'active',
    activeConflicts: 'active conflict(s)',
    addIcsUrl: 'Add ICS URL',
    addSource: 'Add Source',
    all: 'All',
    blockCount: 'block(s)',
    browse: 'Browse',
    calendarName: 'Calendar name',
    calendarSources: 'Calendar Sources',
    clearCache: 'Clear Cache',
    conflicts: 'Conflicts',
    copy: 'Copy',
    copyPreview: 'Copy Preview',
    dashboard: 'Dashboard',
    deepWork: 'deep work',
    deepWorkBlocks: 'Deep Work Blocks',
    delete: 'Delete',
    enabled: 'Enabled',
    everyWorkdayHasDeepWork: 'Every workday has deep work.',
    export: 'Export',
    events: 'events',
    file: 'File',
    focus: 'Focus',
    focusHours: 'Focus Hours',
    focusTarget: 'Focus target',
    fragmentation: 'Fragmentation',
    freeTime: 'Free Time',
    generateAnalysis: 'Generate Analysis',
    healthScore: 'Health Score',
    importIcsFile: 'Import .ics File',
    importType: 'Import type',
    ignore: 'Ignore',
    includeDescriptions: 'Include descriptions',
    includeEventTitles: 'Include event titles',
    includeLocations: 'Include locations',
    includeSourceNames: 'Include source names',
    inRange: 'in range',
    json: 'JSON',
    language: 'Language',
    loadingDashboard: 'Loading dashboard...',
    loadingFocusMetrics: 'Loading focus metrics...',
    loadingReport: 'Loading report...',
    loadingSettings: 'Loading settings...',
    localFile: 'Local file',
    localFilePath: 'Local file path',
    localIcs: 'Local ICS',
    localCacheCleared: 'Local cache cleared',
    longestBlock: 'Longest Block',
    lunchEnd: 'Lunch end',
    lunchStart: 'Lunch start',
    markdown: 'Markdown',
    meetingDensity: 'Meeting Density',
    meetings: 'meetings',
    minFocus: 'Min focus',
    minimumDeepWork: 'Minimum Deep Work',
    name: 'Name',
    nextFocusBlocks: 'Next Focus Blocks',
    noAnalysisReport: 'No analysis report available.',
    noCalendarSources: 'No calendar sources',
    noConflictsMatchFilter: 'No conflicts match the current filter.',
    noFreeBlocks: 'No free blocks inside working hours.',
    noMajorRisks: 'No major risks detected.',
    noFocusDays: 'No Focus Days',
    noOverloadedDays: 'No overloaded days in range.',
    noDeepWorkBlock: 'No deep work block',
    notSynced: 'Not synced',
    overlap: 'Overlap',
    overloadedDays: 'Overloaded Days',
    overloadMinutes: 'Overload minutes',
    preview: 'Preview',
    privacy: 'Privacy',
    range: 'Range',
    refresh: 'Refresh',
    remoteIcs: 'Remote ICS',
    remoteIcsUrl: 'Remote ICS URL',
    remoteUrl: 'Remote URL',
    reports: 'Reports',
    reviewSensitiveDetails: 'Review sensitive details before saving the report.',
    save: 'Save',
    saveReport: 'Save Report',
    settings: 'Settings',
    settingsSaved: 'Settings saved',
    severeOverload: 'Severe overload',
    sourceCount: 'source(s)',
    sources: 'Sources',
    suggestions: 'Suggestions',
    timezone: 'Timezone',
    topRisks: 'Top Risks',
    type: 'Type',
    url: 'URL',
    weeklyOverview: 'Weekly Overview',
    workEnd: 'Work end',
    workStart: 'Work start',
    yourCalendarDataDevice: 'Your calendar data stays on this device by default.',
  },
  zh: {
    active: '活跃',
    activeConflicts: '个活跃冲突',
    addIcsUrl: '添加 ICS URL',
    addSource: '添加来源',
    all: '全部',
    blockCount: '个时段',
    browse: '浏览',
    calendarName: '日历名称',
    calendarSources: '日历来源',
    clearCache: '清除缓存',
    conflicts: '冲突',
    copy: '复制',
    copyPreview: '复制预览',
    dashboard: '仪表盘',
    deepWork: '深度工作',
    deepWorkBlocks: '深度工作时段',
    delete: '删除',
    enabled: '启用',
    everyWorkdayHasDeepWork: '每个工作日都有深度工作时段。',
    export: '导出',
    events: '个事件',
    file: '文件',
    focus: '专注',
    focusHours: '专注小时数',
    focusTarget: '专注目标',
    fragmentation: '碎片化',
    freeTime: '空闲时间',
    generateAnalysis: '生成分析',
    healthScore: '健康评分',
    importIcsFile: '导入 .ics 文件',
    importType: '导入类型',
    ignore: '忽略',
    includeDescriptions: '包含描述',
    includeEventTitles: '包含事件标题',
    includeLocations: '包含地点',
    includeSourceNames: '包含来源名称',
    inRange: '范围内',
    json: 'JSON',
    language: '语言',
    loadingDashboard: '正在加载仪表盘...',
    loadingFocusMetrics: '正在加载专注指标...',
    loadingReport: '正在加载报告...',
    loadingSettings: '正在加载设置...',
    localFile: '本地文件',
    localFilePath: '本地文件路径',
    localIcs: '本地 ICS',
    localCacheCleared: '本地缓存已清除',
    longestBlock: '最长时段',
    lunchEnd: '午餐结束',
    lunchStart: '午餐开始',
    markdown: 'Markdown',
    meetingDensity: '会议密度',
    meetings: '会议',
    minFocus: '最短专注',
    minimumDeepWork: '最短深度工作',
    name: '名称',
    nextFocusBlocks: '接下来的专注时段',
    noAnalysisReport: '没有可用的分析报告。',
    noCalendarSources: '没有日历来源',
    noConflictsMatchFilter: '没有符合当前筛选条件的冲突。',
    noFreeBlocks: '工作时间内没有空闲时段。',
    noMajorRisks: '未检测到主要风险。',
    noFocusDays: '无专注日',
    noOverloadedDays: '范围内没有过载日期。',
    noDeepWorkBlock: '没有深度工作时段',
    notSynced: '未同步',
    overlap: '重叠',
    overloadedDays: '过载日期',
    overloadMinutes: '过载分钟数',
    preview: '预览',
    privacy: '隐私',
    range: '范围',
    refresh: '刷新',
    remoteIcs: '远程 ICS',
    remoteIcsUrl: '远程 ICS URL',
    remoteUrl: '远程 URL',
    reports: '报告',
    reviewSensitiveDetails: '保存报告前请检查敏感细节。',
    save: '保存',
    saveReport: '保存报告',
    settings: '设置',
    settingsSaved: '设置已保存',
    severeOverload: '严重过载',
    sourceCount: '个来源',
    sources: '来源',
    suggestions: '建议',
    timezone: '时区',
    topRisks: '主要风险',
    type: '类型',
    url: 'URL',
    weeklyOverview: '每周概览',
    workEnd: '工作结束',
    workStart: '工作开始',
    yourCalendarDataDevice: '默认情况下，你的日历数据会保留在本设备上。',
  },
} as const;

type TranslationKey = keyof typeof dictionaries.en;

type I18nContextValue = {
  language: Language;
  setLanguage: (language: Language) => void;
  t: (key: TranslationKey) => string;
};

const defaultContext: I18nContextValue = {
  language: 'en',
  setLanguage: () => undefined,
  t: (key) => dictionaries.en[key],
};

const I18nContext = createContext<I18nContextValue>(defaultContext);

export function LanguageProvider({ children }: { children: React.ReactNode }) {
  const [language, setLanguageState] = useState<Language>('en');

  useEffect(() => {
    const stored = window.localStorage.getItem(storageKey);
    if (stored === 'en' || stored === 'zh') {
      setLanguageState(stored);
    }
  }, []);

  useEffect(() => {
    document.documentElement.lang = language === 'zh' ? 'zh-CN' : 'en';
    window.localStorage.setItem(storageKey, language);
  }, [language]);

  const value = useMemo<I18nContextValue>(
    () => ({
      language,
      setLanguage: setLanguageState,
      t: (key) => dictionaries[language][key] ?? dictionaries.en[key],
    }),
    [language],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  return useContext(I18nContext);
}

export function localeForLanguage(language: Language) {
  return language === 'zh' ? 'zh-CN' : 'en-US';
}

export function formatDurationHours(minutes: number, language: Language = 'en') {
  return language === 'zh' ? `${(minutes / 60).toFixed(1)}小时` : `${(minutes / 60).toFixed(1)}h`;
}

export function formatDayCount(days: number, language: Language) {
  return language === 'zh' ? `${days}天` : `${days}d`;
}

const zhTerms: Record<string, string> = {
  All: '全部',
  BoundaryRisk: '边界风险',
  Critical: '严重',
  DeepWork: '深度工作',
  disabled: '已停用',
  Excellent: '优秀',
  failed: '失败',
  Good: '良好',
  High: '高',
  idle: '空闲',
  Low: '低',
  Lunch: '午餐',
  Medium: '中',
  MicroGap: '碎片空档',
  Overloaded: '过载',
  Poor: '较差',
  Risk: '风险',
  SeverelyOverloaded: '严重过载',
  ShortGap: '短空档',
  success: '成功',
  syncing: '同步中',
  Warning: '预警',
};

export function translateTerm(value: string, language: Language) {
  if (language === 'en') return value;
  return zhTerms[value] ?? value;
}

export function translateReportText(value: string, language: Language): string {
  if (language === 'en') return value;

  const pointMatch = value.match(/^(.*) \(([+-]\d+)\)$/);
  if (pointMatch) {
    return `${translateReportText(pointMatch[1], language)} (${pointMatch[2]})`;
  }

  const exact: Record<string, string> = {
    'At least one day has no meetings': '至少有一天没有会议',
    'Busy Event': '忙碌事件',
    'Every workday has a deep work block': '每个工作日都有深度工作时段',
    'Failed to add source': '添加来源失败',
    'Failed to load dashboard': '加载仪表盘失败',
    'Import failed': '导入失败',
    'Local cache cleared': '本地缓存已清除',
    'Meeting time stays below 4h per day': '每天会议时间低于 4 小时',
    'No active calendar sources yet': '还没有启用的日历来源',
    'No active conflicts in the analysis period': '分析周期内没有活跃冲突',
    'No conflicts': '没有冲突',
    'Review a conflicting busy event.': '检查一个冲突的忙碌事件。',
    'Select an .ics file': '请选择 .ics 文件',
    'Settings saved': '设置已保存',
    'Source not found': '未找到来源',
    'Use an HTTP or HTTPS ICS URL': '请使用 HTTP 或 HTTPS ICS URL',
  };

  if (exact[value]) return exact[value];

  let match = value.match(/^(Critical|High|Medium|Low) conflict at (.+)$/);
  if (match) return `${translateTerm(match[1], language)}级冲突：${match[2]}`;

  match = value.match(/^(.+) has no deep work block$/);
  if (match) return `${match[1]} 没有深度工作时段`;

  match = value.match(/^(.+) has more than ([\d.]+)h of meetings$/);
  if (match) return `${match[1]} 的会议超过 ${match[2]} 小时`;

  match = value.match(/^(.+) has a meeting after (.+)$/);
  if (match) return `${match[1]} 有 ${match[2]} 之后的会议`;

  match = value.match(/^(.+) has more than 3 consecutive meetings$/);
  if (match) return `${match[1]} 有超过 3 场连续会议`;

  match = value.match(/^(.+) lunch is covered by meetings$/);
  if (match) return `${match[1]} 的午餐时间被会议占用`;

  match = value.match(/^(.+) has more than 3 micro gaps$/);
  if (match) return `${match[1]} 有超过 3 个碎片空档`;

  match = value.match(/^([\d.]+)h of meetings$/);
  if (match) return `${match[1]} 小时会议`;

  match = value.match(/^more than ([\d.]+)h of meetings$/);
  if (match) return `超过 ${match[1]} 小时会议`;

  match = value.match(/^(\d+) meetings$/);
  if (match) return `${match[1]} 场会议`;

  match = value.match(/^Move (.+) away from (.+)\.?$/);
  if (match) return `将 ${match[1]} 从 ${match[2]} 移开`;

  match = value.match(/^It overlaps with (.+) event\(s\) for (\d+) minutes\.$/);
  if (match) return `它与 ${match[1]} 个事件重叠 ${match[2]} 分钟。`;

  match = value.match(/^It overlaps with (.+) for (\d+) minutes\.$/);
  if (match) return `它与 ${match[1]} 重叠 ${match[2]} 分钟。`;

  match = value.match(/^Protect (.+) as Focus Time\.$/);
  if (match) return `保护 ${match[1]} 作为专注时间。`;

  match = value.match(/^This is a (\d+) minute deep work block\.$/);
  if (match) return `这是一个 ${match[1]} 分钟的深度工作时段。`;

  match = value.match(/^Review buffers on (.+)\.$/);
  if (match) return `检查 ${match[1]} 的缓冲安排。`;

  if (value === 'no deep work block') return '没有深度工作时段';
  if (value === 'more than 3 consecutive meetings') return '超过 3 场连续会议';
  if (value === 'lunch is covered by meetings') return '午餐时间被会议占用';
  if (value === 'meeting after 19:00') return '19:00 之后有会议';

  return value;
}

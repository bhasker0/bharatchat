{{/*
Expand the name of the chart.
*/}}
{{- define "bharatchat.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "bharatchat.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "bharatchat.labels" -}}
helm.sh/chart: {{ include "bharatchat.name" . }}-{{ .Chart.Version | replace "+" "_" }}
{{ include "bharatchat.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
sovereign.in/jurisdiction: {{ .Values.global.jurisdiction | quote }}
sovereign.in/environment: {{ .Values.global.environment | quote }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "bharatchat.selectorLabels" -}}
app.kubernetes.io/name: {{ include "bharatchat.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "bharatchat.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "bharatchat.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

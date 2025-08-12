{{/*
Expand the name of the chart.
*/}}
{{- define "compute-node.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "compute-node.fullname" -}}
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
Create chart name and version as used by the chart label.
*/}}
{{- define "compute-node.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "compute-node.labels" -}}
helm.sh/chart: {{ include "compute-node.chart" . }}
{{ include "compute-node.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "compute-node.selectorLabels" -}}
app.kubernetes.io/name: {{ include "compute-node.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Node-specific labels
*/}}
{{- define "compute-node.nodeLabels" -}}
compute-node/type: {{ include "compute-node.fullname" . }}
{{ include "compute-node.labels" . }}
{{- end }}

{{/*
Node-specific selector labels
*/}}
{{- define "compute-node.nodeSelectorLabels" -}}
compute-node/type: {{ include "compute-node.fullname" . }}
{{ include "compute-node.selectorLabels" . }}
{{- end }}

{{/*
Resource name helpers - all prefixed with compute-node.fullname
*/}}

{{/*
Server StatefulSet name
*/}}
{{- define "compute-node.serverStatefulSetName" -}}
{{- printf "%s-server" (include "compute-node.fullname" .) }}
{{- end }}

{{/*
Worker StatefulSet name
*/}}
{{- define "compute-node.workerStatefulSetName" -}}
{{- printf "%s-worker" (include "compute-node.fullname" .) }}
{{- end }}

{{/*
UI Deployment name
*/}}
{{- define "compute-node.uiDeploymentName" -}}
{{- printf "%s-ui" (include "compute-node.fullname" .) }}
{{- end }}

{{/*
Server Service name
*/}}
{{- define "compute-node.serverServiceName" -}}
{{- printf "%s-server" (include "compute-node.fullname" .) }}
{{- end }}

{{/*
Worker Service name
*/}}
{{- define "compute-node.workerServiceName" -}}
{{- printf "%s-worker" (include "compute-node.fullname" .) }}
{{- end }}

{{/*
UI Service name
*/}}
{{- define "compute-node.uiServiceName" -}}
{{- printf "%s-ui" (include "compute-node.fullname" .) }}
{{- end }}

{{/*
Ingress name
*/}}
{{- define "compute-node.ingressName" -}}
{{- include "compute-node.fullname" . }}
{{- end }}

{{/*
ConfigMap name
*/}}
{{- define "compute-node.configMapName" -}}
{{- include "compute-node.fullname" . }}
{{- end }}

{{/*
Secret name
*/}}
{{- define "compute-node.secretName" -}}
{{- if .Values.security.existingSecretName }}
{{- .Values.security.existingSecretName }}
{{- else }}
{{- include "compute-node.fullname" . }}
{{- end }}
{{- end }}

{{/*
Storage PVC name
*/}}
{{- define "compute-node.storageVolumeName" -}}
{{- printf "%s-data" (include "compute-node.fullname" .) }}
{{- end }}
{{/*
ServiceAccount name
*/}}
{{- define "compute-node.serviceAccountName" -}}
{{- include "compute-node.fullname" . }}
{{- end }}

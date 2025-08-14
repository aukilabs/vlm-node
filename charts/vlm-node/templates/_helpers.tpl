{{/*
Expand the name of the chart.
*/}}
{{- define "vlm-node.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "vlm-node.fullname" -}}
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
{{- define "vlm-node.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "vlm-node.labels" -}}
helm.sh/chart: {{ include "vlm-node.chart" . }}
{{ include "vlm-node.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "vlm-node.selectorLabels" -}}
app.kubernetes.io/name: {{ include "vlm-node.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Node-specific labels
*/}}
{{- define "vlm-node.nodeLabels" -}}
vlm-node/type: {{ include "vlm-node.fullname" . }}
{{ include "vlm-node.labels" . }}
{{- end }}

{{/*
Node-specific selector labels
*/}}
{{- define "vlm-node.nodeSelectorLabels" -}}
vlm-node/type: {{ include "vlm-node.fullname" . }}
{{ include "vlm-node.selectorLabels" . }}
{{- end }}

{{/*
Resource name helpers - all prefixed with vlm-node.fullname
*/}}

{{/*
Server StatefulSet name
*/}}
{{- define "vlm-node.serverStatefulSetName" -}}
{{- printf "%s-server" (include "vlm-node.fullname" .) }}
{{- end }}

{{/*
UI Deployment name
*/}}
{{- define "vlm-node.uiDeploymentName" -}}
{{- printf "%s-ui" (include "vlm-node.fullname" .) }}
{{- end }}

{{/*
Server Service name
*/}}
{{- define "vlm-node.serverServiceName" -}}
{{- printf "%s-server" (include "vlm-node.fullname" .) }}
{{- end }}

{{/*
UI Service name
*/}}
{{- define "vlm-node.uiServiceName" -}}
{{- printf "%s-ui" (include "vlm-node.fullname" .) }}
{{- end }}

{{/*
Ingress name
*/}}
{{- define "vlm-node.ingressName" -}}
{{- include "vlm-node.fullname" . }}
{{- end }}

{{/*
ConfigMap name
*/}}
{{- define "vlm-node.configMapName" -}}
{{- include "vlm-node.fullname" . }}
{{- end }}

{{/*
Secret name
*/}}
{{- define "vlm-node.secretName" -}}
{{- if .Values.security.existingSecretName }}
{{- .Values.security.existingSecretName }}
{{- else }}
{{- include "vlm-node.fullname" . }}
{{- end }}
{{- end }}

{{/*
Storage PVC name
*/}}
{{- define "vlm-node.storageVolumeName" -}}
{{- printf "%s-data" (include "vlm-node.fullname" .) }}
{{- end }}
{{/*
ServiceAccount name
*/}}
{{- define "vlm-node.serviceAccountName" -}}
{{- include "vlm-node.fullname" . }}
{{- end }}

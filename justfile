encode-secrets:
  kubeseal --controller-name=sealed-secrets --controller-namespace=sealed-secrets --context default \
  --format yaml < secrets.yaml > sealed-secrets.yaml



###

Set ACR_NAME
```
    export ACR_NAME=<ACR Name>
    export ACR_RG=<ACR_RG>
```

#### Create and push prodution build

```
cargo build --release
```

Build Image

```
docker build -t ${ACR_NAME}.azurecr.io/az-mock-proxy:1 .
```

Run locally
```
docker run -ti -p 3000:3000 ${ACR_NAME}.azurecr.io/az-mock-proxy:1
```


Push 
```
docker push ${ACR_NAME}.azurecr.io/az-mock-proxy:1
```

#### or just use the cloud (ACR tasks)

```
az acr build --registry $ACR_NAME --image az-mock-proxy:1 -f ./Dockerfile.build .

```


Deploy
```
echo "getting acr creds"
ACR_PASSWD=$(az acr credential show --name ${ACR_NAME} --resource-group ${ACR_RG} --query passwords[0].value --output tsv)
az group create -n az-mock-proxy -l westeurope
sed -e "s|_ACR_PASSWD_|${ACR_PASSWD}|g" -e "s|_ACR_NAME_|${ACR_NAME}|g" -e "s|ACR_PASSWD|${ACR_PASSWD}|g" ./aci_deploy.yml >./aci_final.yml
az container create -l westeurope --resource-group az-mock-proxy -f ./aci_final.yml
```

Call in cloud
```
curl az-mock-proxy.westeurope.azurecontainer.io:3000/api/stock?sku=19278525
```

Show logs
```
az container logs --resource-group az-mock-proxy -n az-mock-proxy --follow
```
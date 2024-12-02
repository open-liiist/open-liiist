import json
import requests
sys.path.append('../')
from libft import to_float

def scraping_shop():
	# Declaration of all information that request.post needs to function correctly

	url = "https://storegabrielli.retailtune.com/store/locator.php"
	user_data = {
		"latitude": 42.8528624,
		"longitude": 13.5389759,
		"lang": "it",
		"radius": 200000
	}
	data = {"user": user_data}
	headers = {"Accept": "*/*"}

	response = requests.post(url, json=data, headers=headers)

	if response.status_code != 200:
		print(f"Error retrieving data: {response.status_code}")
		print(response.text)
		exit()

	shop_data = response.json()

	file_counter = 0
	shop_list = []

	for shop in shop_data:
		info_need = shop.get("store")
		
		if (info_need['click_collect'] == "true") | (info_need['click_drive'] == "true") and "RM" in info_need['address']:
			shop_info = {
				"name" : info_need['name'],
				"street": info_need['address'],
				"lat": to_float(info_need['lat']),
				"long": to_float(info_need['lon']),
				"city": info_need['city'],
				"working_hours": f"{info_need['hours']}",
				"picks_up_in_shop": "True"
				}

			shop_list.append(shop_info)
			file_counter += 1
	# with open(f"store_tigre.json", "w", encoding="utf-8") as outfile:
	# 	json.dump(shop_list, outfile, indent=4)
	return(shop_list)

if __name__ == "__main__":
	scraping_shop()
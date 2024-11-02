import time
import sys
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from utility_tigre import categories_dict

import undetected_chromedriver as uc

# Waits for a single element to appear on the page.
# Returns: The WebElement if found, otherwise None.

def wait_for_element(driver, xpath):

	try:
		element = WebDriverWait(driver, 10).until(
			EC.presence_of_element_located((By.XPATH, xpath))
		)
		if not element:
			print(f"Element not found within 10 seconds: {xpath}")
			return None
		return element
	except Exception as e:
		print(f"Error finding element: {e}")
		driver.quit()
		sys.exit()

# Waits for multiple elements to appear on the page
# Returns: A list of WebElements if found, otherwise an empty list

def wait_for_elements(driver, xpath):

	try:
		elements = driver.find_elements(By.XPATH, xpath)
		if not elements:
			print(f"No elements found in: {xpath}")
			return []
		return elements
	except Exception as e:
		print(f"Error finding elements: {e}")
		driver.quit()
		sys.exit()


# Finds and processes information from product cards in a micro category
# Returns: The number of processed items

def find_and_send_info(driver, n_cards, micro_cate):

	active = 1
	processed_items = 0
	card_selector = f'/html/body/main/div[1]/div[2]/div[2]/div[{micro_cate}]/div/div[2]/div/div/div[1]/div[{{card}}]'

	for card in range(1, n_cards + 1):
		card_xpath = card_selector.format(card=card)

		name = wait_for_element(driver, f"{card_xpath}/div/div[3]/div[2]/h4").text
		image_element = wait_for_element(driver, f"{card_xpath}/div/div[3]/div[1]/a/img")
		img_url = image_element.get_attribute("src")
		description = wait_for_element(driver, f"{card_xpath}/div/div[3]/div[3]/p").text
		new_price = wait_for_element(driver, f"{card_xpath}/div/div[4]/div[1]/div[2]/p").text
		old_price = wait_for_element(driver, f"{card_xpath}/div/div[4]/div[1]/div[1]/p").text

		processed_items += 1
		print(name, description, new_price, old_price)

		if active == 1:
			try:
				wait_for_element(driver, f"/html/body/main/div[1]/div[2]/div[2]/div[{micro_cate}]/div/div[2]/div/div/div[3]").click()
			except:
				active = 0

	return processed_items

# Selects the first Oasi Tigre store in a given location.

def change_shop_location(driver, location):

	driver.get("https://oasitigre.it/it/spesa.html")
	driver.find_element(By.CLASS_NAME, 'ritira-in-negozio-main-page').click()

	wait = WebDriverWait(driver, 20)
	wait.until(EC.visibility_of_element_located((By.CSS_SELECTOR, "input.form-control.pdv.pac-target-input")))

	input_box = driver.find_element(By.CSS_SELECTOR, "input.form-control.pdv.pac-target-input")
	input_box.click()
	input_box.send_keys(location)
	time.sleep(5)
	input_box.send_keys(Keys.ENTER)
	time.sleep(7)

	wait.until(EC.visibility_of_element_located((By.XPATH, "//div[@class='shop-card-container']/div[@class='card-selezione-negozio']")))
	available_shops = driver.find_elements(By.XPATH, "//div[@class='shop-card-container']/div[@class='card-selezione-negozio']")

	if len(available_shops) == 0:
		print('No shops found')
		return

	available_shops[0].click()
	time.sleep(2)
	driver.find_element(By.CLASS_NAME, "scegliDopo").click()
	time.sleep(3)

# Selects the first store in a new location after a previous selection

def change_already_selected_shop(driver, location):

	wait = WebDriverWait(driver, 10)
	wait.until(EC.visibility_of_element_located((By.XPATH, "/html/body/div[1]/div/div/header/div/nav/div/div[3]/div[1]/span[3]")))
	driver.find_element(By.XPATH, "/html/body/div[1]/div/div/header/div/nav/div/div[3]/div[1]/span[3]").click()

	wait.until(EC.visibility_of_element_located((By.XPATH, "/html/body/div[3]/div[2]/div[3]/div[3]/div[1]/div[2]/span[2]")))
	driver.find_element(By.XPATH, "/html/body/div[3]/div[2]/div[3]/div[3]/div[1]/div[2]/span[2]").click()

	wait.until(EC.visibility_of_element_located((By.CSS_SELECTOR, "input.form-control.pdv.pac-target-input")))
	input_box = driver.find_element(By.CSS_SELECTOR, "input.form-control.pdv.pac-target-input")
	input_box.click()
	input_box.send_keys(location)
	time.sleep(5)
	input_box.send_keys(Keys.ENTER)
	time.sleep(7)

	wait.until(EC.visibility_of_element_located((By.XPATH, "//div[@class='shop-card-container']/div[@class='card-selezione-negozio']")))
	available_shops = driver.find_elements(By.XPATH, "//div[@class='shop-card-container']/div[@class='card-selezione-negozio']")

	if len(available_shops) == 0:
		print('No shops found')
		return

	available_shops[0].click()
	time.sleep(2)

if __name__ == "__main__":
	# Initialize WebDriver
	driver = uc.Chrome(use_subprocess=False)

	change_shop_location(driver, "Roma")
	# change_already_selected_shop(driver, "San benedetto del tronto")

	total_items_processed = 0
	for category, items in categories_dict.items():
		for item in items:
			product_url = f"https://oasitigre.it/it/spesa/reparti/{category}/{item}.html"

			driver.get(product_url)
			time.sleep(5)

			n_micro_cate = len(wait_for_elements(driver, '/html/body/main/div[1]/div[2]/div[2]/div'))
			active = 1

			for micro_cate in range(2, n_micro_cate + 1):
				if active == 1:
					try:
						# Scroll to micro category element
						element = wait_for_element(driver, f"/html/body/main/div[1]/div[2]/div[2]/div[{micro_cate}]")
						driver.execute_script('arguments[0].scrollIntoView(true)', element)
					except:
						active = 0

				n_cards = len(wait_for_elements(driver, f'/html/body/main/div[1]/div[2]/div[2]/div[{micro_cate}]/div/div[2]/div/div/div[1]/div'))

				total_items_processed += find_and_send_info(driver, n_cards, micro_cate)

	print(f"Total items processed: {total_items_processed}")

	driver.close()
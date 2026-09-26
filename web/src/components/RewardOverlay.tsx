import { RARITY_LABEL, rewardText } from '../game/labels'
import type { RewardOffer } from '../game/types'

interface RewardOverlayProps {
  offers: RewardOffer[]
  onClaim(index: number): void
}

export function RewardOverlay({ offers, onClaim }: RewardOverlayProps) {
  return (
    <div className="overlay">
      <div className="overlay-card">
        <h2>Objective complete</h2>
        <p className="muted">Pick one reward. The next objective is drawn after it.</p>
        <ul className="offers">
          {offers.map((offer, index) => (
            <li key={index}>
              <button
                type="button"
                className="offer"
                data-rarity={offer.rarity}
                onClick={() => onClaim(index)}
              >
                <span className="offer-rarity">{RARITY_LABEL[offer.rarity]}</span>
                {'Relic' in offer.reward && (
                  <span className="offer-name">{offer.reward.Relic.name}</span>
                )}
                <span className="offer-text">{rewardText(offer.reward)}</span>
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  )
}

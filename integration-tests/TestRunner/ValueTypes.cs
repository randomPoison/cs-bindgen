using Xunit;

namespace TestRunner
{
    public class ValueTypes
    {
        [Fact]
        public void MahjongTile()
        {
            // TODO: Generate a constructor for structs.
            // var tile = new SimpleTile(Suit.Bamboo, 1);
            var tile = new SimpleTile
            {
                Suit = Suit.Bamboo,
                Value = 1,
            };
            Assert.Equal(Suit.Bamboo, tile.Suit);
            Assert.Equal(1, tile.Value);
        }
    }
}
